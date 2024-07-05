use diesel::{expression::AsExpression, pg::Pg, serialize::ToSql};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::serialize::{self, Output};
use diesel::sql_types::Uuid as SqlUuid;


#[derive(Clone, Copy, Debug, Deserialize, Serialize, 
    PartialEq, Eq, PartialOrd, Ord, Hash, AsExpression
)]
#[diesel(sql_type = diesel::sql_types::Uuid)]
#[cfg_attr(features = "query", derive(DieselNewType))]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())  
    }

    pub fn into_inner(self) -> Uuid {
        self.0
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

impl ToSql<SqlUuid, Pg> for UserId {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <Uuid as ToSql<SqlUuid, Pg>>::to_sql(&self.0, out)
    }
}

impl FromSql<SqlUuid, Pg> for UserId {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        <Uuid as FromSql<SqlUuid, Pg>>::from_sql(bytes).map(UserId)
    }
}