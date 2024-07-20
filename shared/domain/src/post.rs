use nutype::nutype;
#[nutype(
    validate(not_empty, len_char_min = 3, len_char_max = 30),
    derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Headline(String);

#[nutype(
    validate(not_empty, len_char_max = 100),
    derive(Clone, Debug, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Message(String);
