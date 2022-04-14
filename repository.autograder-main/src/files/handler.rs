use std::env;
use std::str::FromStr;

use jsonwebtoken::EncodingKey;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use uuid::Uuid;

use crate::auth::Auth;
use crate::config::AppState;
use crate::connection::DbConn;
use crate::files;
use crate::files::{File, ScheduleOutputFile, ScheduleTriggerFile};

#[patch(
    "/<id>/output",
    format = "application/json",
    data = "<schedule_output_file>"
)]
pub fn patch_output(
    auth: Auth,
    id: String,
    schedule_output_file: Json<ScheduleOutputFile>,
    connection: DbConn,
) -> Result<Json<File>, rocket::http::Status> {
    Uuid::from_str(&id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| {
            files::repository::update_output(
                uuid,
                auth,
                schedule_output_file.into_inner(),
                &connection,
            )
            .into()
        })
}

#[patch("/<id>", format = "application/json", data = "<schedule_trigger_file>")]
pub fn patch(
    auth: Auth,
    state: State<AppState>,
    id: String,
    schedule_trigger_file: Json<ScheduleTriggerFile>,
    connection: DbConn,
) -> Result<Json<File>, rocket::http::Status> {
    Uuid::from_str(&id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| {
            let updatable_file = schedule_trigger_file.into_inner();
            let update_result: Result<Json<File>, rocket::http::Status> =
                files::repository::update(uuid, &auth, &updatable_file, &connection).into();
            update_result.map(|file| (file.into_inner(), updatable_file))
        })
        .and_then(|(file, schedule_trigger_file)| {
            schedule_run(
                &auth,
                &state.encoding_key,
                file,
                schedule_trigger_file,
                connection,
            )
        })
}

#[get("/<id>")]
pub fn get(auth: Auth, id: String, connection: DbConn) -> Result<Json<File>, rocket::http::Status> {
    Uuid::from_str(&id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| files::repository::get_by_uuid(uuid, auth, &connection).into())
}

#[get("/?<submission_id>")]
pub fn get_by_submission_id(
    auth: Auth,
    submission_id: String,
    connection: DbConn,
) -> Result<Json<File>, rocket::http::Status> {
    Uuid::from_str(&submission_id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| files::repository::get_by_submission_id(uuid, auth, &connection).into())
}

fn schedule_run(
    auth: &Auth,
    key: &EncodingKey,
    file: File,
    mut schedule_trigger_file: ScheduleTriggerFile,
    connection: DbConn,
) -> Result<Json<File>, Status> {
    env::var("SCHEDULING_SUBMISSION_URL")
        .map(|url| (url, file))
        .map_err(|_| Status::BadRequest)
        .and_then(|(scheduling_url, file)| {
            files::repository::get_schedule_file(file.id, &connection)
                .map(|schedule_file| (scheduling_url, file, schedule_file))
                .map_err(|_| Status::InternalServerError)
        })
        .and_then(|(scheduling_url, file, schedule_file)| {
            reqwest::blocking::Client::new()
                .post(&scheduling_url)
                .headers(construct_headers())
                .bearer_auth(auth.token(key))
                .json(&schedule_file)
                .send()
                .map(|_| Json(file))
                .map_err(|_| {
                    schedule_trigger_file.scheduled = false;
                    let update_result: Result<Json<File>, rocket::http::Status> =
                        files::repository::update(
                            schedule_trigger_file.id,
                            auth,
                            &schedule_trigger_file,
                            &connection,
                        )
                        .into();
                    update_result.map_or(Status::NotFound, |_| Status::InternalServerError)
                })
        })
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}
