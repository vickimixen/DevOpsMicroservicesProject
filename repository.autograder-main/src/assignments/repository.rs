use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::assignments::{Assignment, UpdatableAssignment};
use crate::auth::Auth;
use crate::connection::{RepositoryError, RepositoryQueryResult};
use crate::schema::assignments;

pub fn insert(
    assignment: Assignment,
    auth: Auth,
    connection: &PgConnection,
) -> RepositoryQueryResult<Assignment> {
    if auth.is_student {
        return RepositoryQueryResult::Err(RepositoryError::Unauthorized(
            "student can't create assignment".to_owned(),
        ));
    }
    diesel::insert_into(assignments::table)
        .values(&assignment)
        .get_result(connection)
        .into()
}

pub fn get(id: Uuid, auth: Auth, connection: &PgConnection) -> RepositoryQueryResult<Assignment> {
    // TODO this can be refactored with a call to the assignment service. If an assignment is public,
    // it can be viewed by anyone. If not, by the teacher, superuser and the affiliated students.
    match is_allowed_to_change(id, auth, connection) {
        Ok(false) => {
            return RepositoryQueryResult::Err(RepositoryError::Unauthorized(
                "user can't see assignment".to_owned(),
            ))
        }
        Err(_) => return RepositoryQueryResult::Err(RepositoryError::NotFound),
        _ => {}
    }
    assignments::table
        .find(id)
        .get_result::<Assignment>(connection)
        .into()
}

pub fn is_allowed_to_change(id: Uuid, auth: Auth, connection: &PgConnection) -> QueryResult<bool> {
    if auth.is_superuser {
        return QueryResult::Ok(true);
    }
    assignments::table
        .find(id)
        .get_result::<Assignment>(connection)
        .map(|assignment| assignment.user_id == auth.user_id)
}

pub fn update(
    id: Uuid,
    auth: Auth,
    updatable_assignment: UpdatableAssignment,
    connection: &PgConnection,
) -> RepositoryQueryResult<Assignment> {
    match is_allowed_to_change(id, auth, connection) {
        Ok(false) => {
            return RepositoryQueryResult::Err(RepositoryError::Unauthorized(
                "user can't update assignment".to_owned(),
            ))
        }
        Err(_) => return RepositoryQueryResult::Err(RepositoryError::NotFound),
        _ => {}
    }
    let entry = diesel::update(assignments::table.find(id));
    let time = Utc::now().naive_utc();
    match (
        updatable_assignment.encoded_input,
        updatable_assignment.encoded_output,
    ) {
        (Some(encoded_input), Some(encoded_output)) => entry
            .set((
                assignments::encoded_output.eq(encoded_output),
                assignments::encoded_input.eq(encoded_input),
                assignments::updated.eq(time),
            ))
            .get_result(connection),
        (Some(encoded_input), None) => entry
            .set((
                assignments::encoded_input.eq(encoded_input),
                assignments::updated.eq(time),
            ))
            .get_result(connection),
        (None, Some(encoded_output)) => entry
            .set((
                assignments::encoded_output.eq(encoded_output),
                assignments::updated.eq(time),
            ))
            .get_result(connection),
        _ => assignments::table.find(id).get_result(connection),
    }
    .into()
}
