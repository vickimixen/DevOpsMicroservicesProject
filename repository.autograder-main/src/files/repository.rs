use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::auth::Auth;
use crate::connection::RepositoryError::{NotFound, Unauthorized};
use crate::connection::RepositoryQueryResult;
use crate::files::repository::Ownership::Viewer;
use crate::files::{
    File, InsertableFile, ScheduleInputFile, ScheduleOutputFile, ScheduleTriggerFile,
    ValidatableFile, ValidatedFile,
};
use crate::schema::{assignments, files, submissions};

enum Ownership {
    Owner,
    Viewer,
    Unauthorized,
}

#[derive(Queryable)]
struct Owners {
    owner_id: Uuid,
    assignment_owner_id: Uuid,
}

impl Owners {
    fn ownership_by_id(&self, user_id: Uuid) -> Ownership {
        match (user_id, self.assignment_owner_id, self.owner_id) {
            (input, ass, _) if input == ass => Ownership::Owner,
            (input, _, own) if input == own => Ownership::Viewer,
            (_, _, _) => Ownership::Unauthorized,
        }
    }
}

pub fn insert(insertable_file: InsertableFile, connection: &PgConnection) -> QueryResult<File> {
    diesel::insert_into(files::table)
        .values(&insertable_file)
        .get_result(connection)
}

pub fn update(
    id: Uuid,
    auth: &Auth,
    schedule_trigger_file: &ScheduleTriggerFile,
    connection: &PgConnection,
) -> RepositoryQueryResult<File> {
    match ownership_of(id, auth, connection) {
        Ok(Ownership::Unauthorized) if !auth.is_superuser => RepositoryQueryResult::Err(
            Unauthorized("can't update file if not owner of file".to_owned()),
        ),
        Ok(_) => diesel::update(files::table.find(id))
            .set((
                schedule_trigger_file,
                files::updated.eq(Utc::now().naive_utc()),
            ))
            .get_result(connection)
            .into(),
        Err(_) => RepositoryQueryResult::Err(NotFound),
    }
}

pub fn update_output(
    id: Uuid,
    auth: Auth,
    schedule_output_file: ScheduleOutputFile,
    connection: &PgConnection,
) -> RepositoryQueryResult<File> {
    match ownership_of(id, &auth, connection) {
        Ok(Ownership::Unauthorized) if !auth.is_superuser => RepositoryQueryResult::Err(
            Unauthorized("can't update file if not owner of file".to_owned()),
        ),
        Ok(_) => patch_schedule_output(id, schedule_output_file, connection).into(),
        Err(_) => RepositoryQueryResult::Err(NotFound),
    }
}

fn patch_schedule_output(
    id: Uuid,
    schedule_output_file: ScheduleOutputFile,
    connection: &PgConnection,
) -> QueryResult<File> {
    let result: QueryResult<ValidatedFile> = files::table
        .find(id)
        .inner_join(submissions::table.inner_join(assignments::table))
        .select((
            files::id,
            files::validated,
            files::encoded_output,
            assignments::encoded_output,
            files::updated,
        ))
        .first(connection)
        .map(|mut validatable_file: ValidatableFile| {
            validatable_file.encoded_output = Some(schedule_output_file.encoded_output);
            validatable_file.updated = Utc::now().naive_utc();
            ValidatedFile::from(validatable_file)
        });

    result.and_then(|validatable_file| {
        diesel::update(files::table.find(validatable_file.id))
            .set(validatable_file)
            .get_result(connection)
    })
}

pub fn get_by_uuid(id: Uuid, auth: Auth, connection: &PgConnection) -> RepositoryQueryResult<File> {
    match ownership_of(id, &auth, connection) {
        Ok(Ownership::Unauthorized) if !auth.is_superuser => {
            RepositoryQueryResult::Err(Unauthorized("user is not allowed to query file".to_owned()))
        }
        Ok(_) => files::table.find(id).get_result(connection).into(),
        Err(_) => RepositoryQueryResult::Err(NotFound),
    }
}

pub fn get_schedule_file(id: Uuid, connection: &PgConnection) -> QueryResult<ScheduleInputFile> {
    files::table
        .inner_join(submissions::table.inner_join(assignments::table))
        .filter(files::id.eq(id))
        .select((
            files::id,
            submissions::extension,
            submissions::assignment_id,
            files::encoded_text,
            assignments::encoded_input,
        ))
        .first(connection)
}

fn ownership_of(id: Uuid, auth: &Auth, connection: &PgConnection) -> QueryResult<Ownership> {
    if auth.is_superuser {
        return QueryResult::Ok(Viewer);
    }
    let owners_result: QueryResult<Owners> = files::table
        .find(id)
        .inner_join(submissions::table.inner_join(assignments::table))
        .select((submissions::user_id, assignments::user_id))
        .get_result(connection);
    owners_result.map(|owners| owners.ownership_by_id(auth.user_id))
}

pub fn get_by_submission_id(
    submission_id: Uuid,
    auth: Auth,
    connection: &PgConnection,
) -> RepositoryQueryResult<File> {
    let result: QueryResult<File> = files::table
        .filter(files::submission_id.eq(submission_id))
        .order(files::updated.desc())
        .first(connection);

    match result {
        Ok(file) => match ownership_of(file.id, &auth, connection) {
            Ok(Ownership::Unauthorized) if !auth.is_superuser => RepositoryQueryResult::Err(
                Unauthorized("user is not allowed to query file".to_owned()),
            ),
            Ok(_) => RepositoryQueryResult::Ok(file),
            Err(_) => RepositoryQueryResult::Err(NotFound),
        },
        Err(_) => RepositoryQueryResult::Err(NotFound),
    }
}
