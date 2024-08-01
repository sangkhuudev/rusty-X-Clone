use nutype::nutype;

#[nutype(
    validate(not_empty, len_char_max = 50),
    derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct PollHeadline(String);

impl PollHeadline {
    pub const MAX_CHARS: usize = 50;
}

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

#[nutype(
    validate(not_empty, len_char_max = 60),
    derive(Clone, Debug, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Caption(String);

impl Caption {
    pub const MAX_CHARS: usize = 60;
}

#[nutype(
    validate(not_empty, len_char_max = 80),
    derive(Clone, Debug, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct PollChoiceDescription(String);

impl PollChoiceDescription {
    pub const MAX_CHARS: usize = 80;
}
