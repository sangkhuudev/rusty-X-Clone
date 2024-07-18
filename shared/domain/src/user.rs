use nutype::nutype;
#[nutype(
    validate(not_empty, len_char_min = 3, len_char_max = 20),
    derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Username(String);

#[nutype(
    validate(not_empty, len_char_min = 8),
    derive(Clone, Debug, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Password(String);
