use nutype::nutype;

#[nutype(
    validate(len_char_min = 3, len_char_max = 10),
    derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Username(String);

#[nutype(
    validate(len_char_min = 8),
    derive(Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Password(String);
