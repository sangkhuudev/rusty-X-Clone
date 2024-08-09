use nutype::nutype;
use regex::Regex;
use once_cell::sync::Lazy;

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

#[nutype(
    validate(len_char_max = 20),
    derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct DisplayName(String);

impl DisplayName {
    pub const MAX_CHARS: usize = 30;
}

//---------------------------------------------------------------
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(
    || Regex::new(r#"^\S+@\S+\.\S{1,64}$"#).unwrap());


#[nutype(
    validate(regex = EMAIL_REGEX),
    derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, AsRef)
)]
pub struct Email(String);

