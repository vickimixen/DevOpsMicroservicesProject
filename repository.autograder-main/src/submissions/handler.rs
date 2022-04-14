use std::str::FromStr;

use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;
use uuid::Uuid;

use connection::DbConn;

use crate::auth::Auth;
use crate::connection::to_status_created;
use crate::files;
use crate::files::InsertableFile;
use crate::submissions;
use crate::submissions::{InsertableCode, InsertableSubmission, Submission, SubmissionWithFile};

use super::super::connection;

#[get("/", rank = 3)]
pub fn all(auth: Auth, connection: DbConn) -> Result<Json<Vec<Submission>>, rocket::http::Status> {
    submissions::repository::all(auth, &connection).into()
}

#[get("/?<user_id>&<assignment_id>", rank = 1)]
pub fn get_by_unique(
    auth: Auth,
    user_id: String,
    assignment_id: String,
    connection: DbConn,
) -> Result<Json<Submission>, rocket::http::Status> {
    Uuid::from_str(&user_id)
        .map_err(|_| Status::BadRequest)
        .and_then(|user_id| {
            Uuid::from_str(&assignment_id)
                .map(|assignment_id| (user_id, assignment_id))
                .map_err(|_| Status::BadRequest)
        })
        .and_then(|(user_id, assignment_id)| {
            submissions::repository::get_by_unique(assignment_id, user_id, auth, &connection).into()
        })
}

#[get("/?<assignment_id>", rank = 2)]
pub fn all_submissions_for_assignment(
    auth: Auth,
    assignment_id: String,
    connection: DbConn,
) -> Result<Json<Vec<Submission>>, rocket::http::Status> {
    Uuid::from_str(&assignment_id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| {
            submissions::repository::all_by_assignment_id(uuid, auth, &connection).into()
        })
}

#[post("/", format = "application/json", data = "<insertable_code>")]
pub fn insert(
    auth: Auth,
    insertable_code: Json<InsertableCode>,
    connection: DbConn,
) -> Result<status::Created<Json<SubmissionWithFile>>, rocket::http::Status> {
    let insertable_code_parsed = insertable_code.into_inner();
    let result: Result<Json<Submission>, rocket::http::Status> = submissions::repository::insert(
        InsertableSubmission::from(&insertable_code_parsed),
        auth,
        &connection,
    )
    .into();
    result
        .map(|submission| submission.into_inner())
        .map_err(|_| Status::BadRequest)
        .and_then(|submission| {
            InsertableFile::from_insertable_code(&submission, &insertable_code_parsed)
                .map(|insertable_file| (submission, insertable_file))
                .map_err(|_| Status::BadRequest)
        })
        .and_then(|(submission, insertable_file)| {
            files::repository::insert(insertable_file, &connection)
                .map(|file| (submission, file))
                .map_err(|_| Status::BadRequest)
        })
        .map(SubmissionWithFile::from)
        .map(Json)
        .map(|submission_with_file| {
            to_status_created(
                submission_with_file.file_id,
                "/files/",
                submission_with_file,
            )
        })
}
