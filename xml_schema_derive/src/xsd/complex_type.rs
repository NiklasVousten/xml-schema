use crate::xsd::{
  annotation::Annotation, attribute::Attribute, complex_content::ComplexContent,
  sequence::Sequence, simple_content::SimpleContent, Implementation, XsdContext,
};
use heck::ToUpperCamelCase;
use proc_macro2::{Span, TokenStream};
use syn::Ident;

#[derive(Clone, Default, Debug, PartialEq, YaDeserialize)]
#[yaserde(
  rename = "complexType"
  prefix = "xs",
  namespace = "xs: http://www.w3.org/2001/XMLSchema"
)]
pub struct ComplexType {
  #[yaserde(attribute)]
  pub name: String,
  #[yaserde(rename = "attribute")]
  pub attributes: Vec<Attribute>,
  pub sequence: Option<Sequence>,
  #[yaserde(rename = "simpleContent")]
  pub simple_content: Option<SimpleContent>,
  #[yaserde(rename = "complexContent")]
  pub complex_content: Option<ComplexContent>,
  #[yaserde(rename = "annotation")]
  pub annotation: Option<Annotation>,
}

impl Implementation for ComplexType {
  fn implement(
    &self,
    namespace_definition: &TokenStream,
    prefix: &Option<String>,
    context: &XsdContext,
  ) -> TokenStream {
    let struct_name = Ident::new(
      &self.name.replace('.', "_").to_upper_camel_case(),
      Span::call_site(),
    );
    log::info!("Generate sequence");

    let mut binding = self.sequence.clone();
    let self_sequence = if let Some(sequence) = &mut binding {
      for element in &mut sequence.elements {
        if element.kind.is_some() && self.name == element.kind.clone().unwrap() {
          element.recursive = true;
        }
      }
      Some(sequence)
    } else {
      None
    };

    let sequence = self_sequence
      .as_ref()
      .map(|sequence| sequence.implement(namespace_definition, prefix, context))
      .unwrap_or_default();

    log::info!("Generate simple content");
    let simple_content = self
      .simple_content
      .as_ref()
      .map(|simple_content| simple_content.implement(namespace_definition, prefix, context))
      .unwrap_or_default();

    let complex_content = self
      .complex_content
      .as_ref()
      .map(|complex_content| {
        let complex_content_type = complex_content.get_field_implementation(context, prefix);
        quote!(
          #[yaserde(flatten)]
          #complex_content_type,
        )
      })
      .unwrap_or_default();

    let attributes: TokenStream = self
      .attributes
      .iter()
      .map(|attribute| attribute.implement(namespace_definition, prefix, context))
      .collect();

    let sub_types_implementation = self_sequence
      .as_ref()
      .map(|sequence| sequence.get_sub_types_implementation(context, namespace_definition, prefix))
      .unwrap_or_default();

    let docs = self
      .annotation
      .as_ref()
      .map(|annotation| annotation.implement(namespace_definition, prefix, context))
      .unwrap_or_default();

    quote! {
      #docs

      #[derive(Clone, Debug, Default, PartialEq, yaserde_derive::YaDeserialize, yaserde_derive::YaSerialize)]
      #namespace_definition
      pub struct #struct_name {
        #sequence
        #simple_content
        #complex_content
        #attributes
      }

      #sub_types_implementation
    }
  }
}

impl ComplexType {
  pub fn get_field_implementation(
    &self,
    context: &XsdContext,
    prefix: &Option<String>,
  ) -> TokenStream {
    if self.sequence.is_some() {
      self
        .sequence
        .as_ref()
        .map(|sequence| sequence.get_field_implementation(context, prefix))
        .unwrap_or_default()
    } else {
      self
        .simple_content
        .as_ref()
        .map(|simple_content| simple_content.get_field_implementation(context, prefix))
        .unwrap_or_default()
    }
  }

  pub fn get_integrated_implementation(&self, parent_name: &str) -> TokenStream {
    if self.simple_content.is_some() {
      return quote!(String);
    }

    if self.sequence.is_some() {
      let list_wrapper = Ident::new(parent_name, Span::call_site());
      return quote!(#list_wrapper);
    }

    quote!(String)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn recursive() {
    use crate::xsd::element::Element;

    let st = ComplexType {
      name: "recursive".to_string(),
      annotation: None,
      attributes: vec![],
      complex_content: None,
      simple_content: None,
      sequence: Some(Sequence {
        elements: vec![Element {
          name: "next".to_string(),
          annotation: None,
          kind: Some("recursive".to_string()),
          simple_type: None,
          complex_type: None,
          refers: None,
          min_occurences: None,
          max_occurences: None,
          recursive: false, //Will be set to true by the complex type
        }],
      }),
    };

    let context =
      XsdContext::new(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"></xs:schema>"#)
        .unwrap();

    let implementation = st.implement(&TokenStream::new(), &None, &context);

    let expected = TokenStream::from_str(
      "
        # [derive (Clone , Debug , Default , PartialEq , yaserde_derive :: YaDeserialize , yaserde_derive :: YaSerialize)]
        pub struct Recursive {
          # [yaserde (rename = \"next\")]
          pub next : Box< xml_schema_types :: Recursive > ,
        }
      ",
    )
    .unwrap();

    assert_eq!(implementation.to_string(), expected.to_string());
  }
}
