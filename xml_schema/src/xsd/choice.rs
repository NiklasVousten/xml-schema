use crate::xsd::{
  element::Element, group::Group, sequence::Sequence, MaxOccurences,
};

#[derive(Clone, Default, Debug, PartialEq, YaDeserialize)]
#[yaserde(
  root = "choice",
  prefix = "xs",
  namespace = "xs: http://www.w3.org/2001/XMLSchema"
)]
pub struct Choice {
  #[yaserde(attribute)]
  pub name: String,
  #[yaserde(rename = "ref", attribute)]
  pub refers: Option<String>,
  #[yaserde(rename = "minOccurs", attribute)]
  pub min_occurences: Option<u64>,
  #[yaserde(rename = "maxOccurs", attribute)]
  pub max_occurences: Option<MaxOccurences>,

  
  #[yaserde(rename = "element")]
  pub elements: Vec<Element>,

  #[yaserde(rename = "group")]
  pub groups: Vec<Group>,

  //Leads to problems
  /*
  #[yaserde(rename = "choice")]
  pub choices: Vec<Choice>,
  */

  #[yaserde(rename = "sequence")]
  pub sequences: Vec<Sequence>,

    //#[yaserde(text)]
    //pub entries: Vec<ChoiceEntry>
}

#[derive(Clone, Debug, Default, PartialEq, YaDeserialize)]
pub enum ChoiceEntry {
    #[yaserde(rename = "element")]
    Element(Element),

    #[yaserde(rename = "group")]
    Group(Group),

    #[yaserde(rename = "choice")]
    Choice(Choice),

    #[yaserde(rename = "sequence")]
    Sequence(Sequence),

    #[yaserde(rename = "any")]
    #[default]
    Any,
}

#[cfg(test)]
mod tests {
// Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_deserialize_choice() {
    let w3school_example = "
            <xs:choice>
                <xs:element name=\"employee\" type=\"employee\"/>
                <xs:element name=\"member\" type=\"member\"/>
            </xs:choice>
        ";

    let choice_result = yaserde::de::from_str::<Choice>(&w3school_example.replace("xs:", ""));

    assert!(choice_result.is_ok());

    let choice = choice_result.unwrap();

    assert_eq!(choice.elements.len(), 2);
    assert!(choice.elements.iter().any(|elem| elem.name == "employee"));
    assert!(choice.elements.iter().any(|elem| elem.name == "member"));
  }
}
