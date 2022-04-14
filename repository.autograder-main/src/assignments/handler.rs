use std::str::FromStr;

use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;
use uuid::Uuid;

use crate::assignments;
use crate::assignments::{Assignment, InsertableAssignment, UpdatableAssignment};
use crate::auth::Auth;
use crate::connection::{to_status_created, DbConn};

#[post("/", format = "application/json", data = "<insertable_assignment>")]
pub fn insert(
    auth: Auth,
    insertable_assignment: Json<InsertableAssignment>,
    connection: DbConn,
) -> Result<status::Created<Json<Assignment>>, rocket::http::Status> {
    let result = assignments::repository::insert(
        insertable_assignment.into_inner().into(),
        auth,
        &connection,
    );
    let result: Result<Json<Assignment>, rocket::http::Status> = result.into();
    result.map(|assignment| to_status_created(assignment.id, "/assignment/", assignment))
}

#[patch("/<id>", format = "application/json", data = "<updatable_assignment>")]
pub fn update(
    auth: Auth,
    id: String,
    updatable_assignment: Json<UpdatableAssignment>,
    connection: DbConn,
) -> Result<Json<Assignment>, rocket::http::Status> {
    Uuid::from_str(&id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| {
            assignments::repository::update(
                uuid,
                auth,
                updatable_assignment.into_inner(),
                &connection,
            )
            .into()
        })
}

#[get("/<id>")]
pub fn get(
    auth: Auth,
    id: String,
    connection: DbConn,
) -> Result<Json<Assignment>, rocket::http::Status> {
    Uuid::from_str(&id)
        .map_err(|_| Status::BadRequest)
        .and_then(|uuid| assignments::repository::get(uuid, auth, &connection).into())
}
