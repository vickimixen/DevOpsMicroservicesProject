use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

use crate::connection::{deserialize_base64, deserialize_optional_base64, serialize_base64};

use super::schema::assignments;

pub mod handler;
pub mod repository;
pub mod router;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Serialize, Insertable)]
pub struct Assignment {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(serialize_with = "serialize_base64")]
    pub encoded_input: Vec<u8>,
    #[serde(serialize_with = "serialize_base64")]
    pub encoded_output: Vec<u8>,
    pub updated: NaiveDateTime,
}

#[derive(Deserialize, Insertable)]
#[table_name = "assignments"]
pub struct InsertableAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(deserialize_with = "deserialize_base64")]
    pub encoded_input: Vec<u8>,
    #[serde(deserialize_with = "deserialize_base64")]
    pub encoded_output: Vec<u8>,
}

#[derive(Deserialize)]
pub struct UpdatableAssignment {
    #[serde(deserialize_with = "deserialize_optional_base64")]
    #[serde(default)]
    pub encoded_input: Option<Vec<u8>>,
    #[serde(deserialize_with = "deserialize_optional_base64")]
    #[serde(default)]
    pub encoded_output: Option<Vec<u8>>,
}

impl From<InsertableAssignment> for Assignment {
    fn from(insertable_assignment: InsertableAssignment) -> Assignment {
        Assignment {
            id: insertable_assignment.id,
            encoded_output: insertable_assignment.encoded_output,
            encoded_input: insertable_assignment.encoded_input,
            updated: Utc::now().naive_utc(),
            user_id: insertable_assignment.user_id,
        }
    }
}
