use nutype::nutype;
#[nutype(
    validate(not_empty, len_char_max = 30),
    derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Headline(String);

impl Headline {
    pub const MAX_CHARS: usize = 30;
}

#[nutype(
    validate(not_empty, len_char_max = 100),
    derive(Clone, Debug, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Message(String);

impl Message {
    pub const MAX_CHARS: usize = 100;
}