use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, 
    PartialEq, Eq, PartialOrd, Ord, Hash
)]
#[cfg_attr(feature = "query", derive())]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())  
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::str::FromStr for UserId {
    type Err =  IdError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Uuid::try_parse(id)
        .map(|id| id.into())
        .map_err(|_| IdError::Parse)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("Failed to parse id")]
    Parse
}