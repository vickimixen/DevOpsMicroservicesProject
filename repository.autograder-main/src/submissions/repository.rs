use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::assignments::Assignment;
use crate::auth::Auth;
use crate::connection::RepositoryError::{NotFound, Unauthorized};
use crate::connection::{RepositoryError, RepositoryQueryResult, RepositoryResult};
use crate::schema::{assignments, submissions};
use crate::submissions::{InsertableSubmission, Submission};

pub fn all(auth: Auth, connection: &PgConnection) -> RepositoryQueryResult<Vec<Submission>> {
    if !auth.is_superuser {
        return RepositoryQueryResult::Err(RepositoryError::Unauthorized(
            "user is no superuser".to_owned(),
        ));
    }
    submissions::table
        .order(submissions::created.desc())
        .load::<Submission>(connection)
        .into()
}

pub fn all_by_assignment_id(
    assignment_id: Uuid,
    auth: Auth,
    connection: &PgConnection,
) -> RepositoryQueryResult<Vec<Submission>> {
    let assignment_result: QueryResult<Assignment> = assignments::table
        .find(assignment_id)
        .get_result(connection);
    match assignment_result {
        Ok(assignment) => {
            if !(assignment.user_id == auth.user_id || auth.is_superuser) {
                return RepositoryQueryResult::Err(Unauthorized(
                    "user can't view assignment".to_owned(),
                ));
            }
            submissions::table
                .order(submissions::created.desc())
                .filter(submissions::assignment_id.eq(assignment_id))
                .load::<Submission>(connection)
                .into()
        }
        Err(_) => RepositoryQueryResult::Err(NotFound),
    }
}

pub fn insert(
    insertable_submission: InsertableSubmission,
    auth: Auth,
    connection: &PgConnection,
) -> RepositoryQueryResult<Submission> {
    match get_by_unique(
        insertable_submission.assignment_id,
        insertable_submission.user_id,
        auth,
        connection,
    ) {
        RepositoryResult::Ok(mut submission) => {
            submission.created = Utc::now().naive_utc();
            submission.update_count += 1;
            diesel::update(submissions::table.find(submission.id))
                .set(&submission)
                .get_result(connection)
                .into()
        }
        RepositoryResult::Err(error) => match error {
            Unauthorized(_) => {
                RepositoryQueryResult::Err(Unauthorized("user can't insert submission".to_owned()))
            }
            _ => diesel::insert_into(submissions::table)
                .values(&insertable_submission)
                .get_result(connection)
                .into(),
        },
    }
}

pub fn get_by_unique(
    assignment_id: Uuid,
    user_id: Uuid,
    auth: Auth,
    connection: &PgConnection,
) -> RepositoryQueryResult<Submission> {
    if !(auth.user_id == user_id || auth.is_superuser) {
        // TODO reafactor with call to assignment service and check if able to create assignment
        return RepositoryQueryResult::Err(RepositoryError::Unauthorized(
            "user is not allowed to query file".to_owned(),
        ));
    }
    submissions::table
        .filter(submissions::assignment_id.eq(assignment_id))
        .filter(submissions::user_id.eq(user_id))
        .first(connection)
        .into()
}
