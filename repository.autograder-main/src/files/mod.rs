use base64::DecodeError;
use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;

use crate::connection::{
    deserialize_base64, deserialize_optional_base64, serialize_base64, serialize_optional_base64,
};
use crate::submissions::InsertableCode;

use super::schema::files;
use super::submissions::Submission;

pub mod handler;
pub mod repository;
pub mod router;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Serialize, Deserialize)]
#[belongs_to(Submission)]
pub struct File {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub updated: NaiveDateTime,
    #[serde(serialize_with = "serialize_base64")]
    pub encoded_text: Vec<u8>,
    pub scheduled: bool,
    pub validated: bool,
    #[serde(serialize_with = "serialize_optional_base64")]
    #[serde(deserialize_with = "deserialize_optional_base64")]
    #[serde(default)]
    pub encoded_output: Option<Vec<u8>>,
}

#[derive(Insertable, Associations)]
#[table_name = "files"]
#[belongs_to(Submission)]
pub struct InsertableFile {
    pub submission_id: Uuid,
    pub encoded_text: Vec<u8>,
}

#[derive(Identifiable, Queryable, Deserialize, AsChangeset, Copy, Clone)]
#[table_name = "files"]
pub struct ScheduleTriggerFile {
    pub id: Uuid,
    pub scheduled: bool,
}

#[derive(Deserialize, AsChangeset)]
#[table_name = "files"]
pub struct ScheduleOutputFile {
    #[serde(deserialize_with = "deserialize_base64")]
    pub encoded_output: Vec<u8>,
}

#[derive(Queryable)]
pub struct ValidatableFile {
    pub id: Uuid,
    pub validated: bool,
    pub encoded_output: Option<Vec<u8>>,
    pub encoded_sample_output: Vec<u8>,
    pub updated: NaiveDateTime,
}

impl ValidatableFile {
    pub fn is_valid(&self) -> bool {
        match &self.encoded_output {
            None => false,
            Some(encoded_output) => self.encoded_sample_output.eq(encoded_output),
        }
    }
}

#[derive(Identifiable, AsChangeset)]
#[table_name = "files"]
pub struct ValidatedFile {
    pub id: Uuid,
    pub validated: bool,
    pub encoded_output: Option<Vec<u8>>,
    pub updated: NaiveDateTime,
}

impl From<ValidatableFile> for ValidatedFile {
    fn from(validatable_file: ValidatableFile) -> ValidatedFile {
        ValidatedFile {
            id: validatable_file.id,
            validated: validatable_file.is_valid(),
            encoded_output: validatable_file.encoded_output,
            updated: validatable_file.updated,
        }
    }
}

#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "files"]
pub struct ScheduleInputFile {
    #[serde(rename(serialize = "file_id"))]
    pub id: Uuid,
    pub extension: String,
    pub assignment_id: Uuid,
    #[serde(rename(serialize = "content"))]
    #[serde(serialize_with = "serialize_base64")]
    pub encoded_text: Vec<u8>,
    #[serde(rename(serialize = "test_case"))]
    #[serde(serialize_with = "serialize_base64")]
    pub encoded_input: Vec<u8>,
}

impl InsertableFile {
    pub fn from_insertable_code(
        submission: &Submission,
        insertable_code: &InsertableCode,
    ) -> Result<InsertableFile, DecodeError> {
        base64::decode(&insertable_code.encoded_text).map(|value| InsertableFile {
            submission_id: submission.id,
            encoded_text: value,
        })
    }
}

impl From<File> for ScheduleTriggerFile {
    fn from(file: File) -> Self {
        ScheduleTriggerFile {
            id: file.id,
            scheduled: file.scheduled,
        }
    }
}
