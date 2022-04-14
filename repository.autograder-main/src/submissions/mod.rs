use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::files::File;

use super::schema::submissions;

pub mod handler;
pub mod repository;
pub mod router;

#[derive(
    Queryable, AsChangeset, Serialize, Deserialize, Identifiable, PartialEq, Debug, Insertable,
)]
pub struct Submission {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub user_id: Uuid,
    pub extension: String,
    pub created: NaiveDateTime,
    pub update_count: i16,
}

#[derive(Serialize, Identifiable)]
#[table_name = "submissions"]
pub struct SubmissionWithFile {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub user_id: Uuid,
    pub extension: String,
    pub update_count: i16,
    pub file_id: Uuid,
}

impl From<(Submission, File)> for SubmissionWithFile {
    fn from(file_and_sub: (Submission, File)) -> Self {
        SubmissionWithFile {
            id: file_and_sub.0.id,
            assignment_id: file_and_sub.0.assignment_id,
            user_id: file_and_sub.0.user_id,
            extension: file_and_sub.0.extension.clone(),
            update_count: file_and_sub.0.update_count,
            file_id: file_and_sub.1.id,
        }
    }
}

#[derive(Deserialize)]
pub struct InsertableCode {
    pub assignment_id: Uuid,
    pub user_id: Uuid,
    pub extension: String,
    pub encoded_text: String,
}

#[derive(Insertable)]
#[table_name = "submissions"]
pub struct InsertableSubmission {
    pub assignment_id: Uuid,
    pub user_id: Uuid,
    pub extension: String,
}

impl From<&InsertableCode> for InsertableSubmission {
    fn from(insertable_code: &InsertableCode) -> Self {
        InsertableSubmission {
            assignment_id: insertable_code.assignment_id,
            user_id: insertable_code.user_id,
            extension: insertable_code.extension.clone(),
        }
    }
}
