use std::env;
use std::ops::Deref;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::QueryResult;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::response::status;
use rocket::{Outcome, Request, State};
use rocket_contrib::json::Json;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};
use uuid::Uuid;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

embed_migrations!();

pub fn init_pool() -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url());
    let pool = Pool::new(manager).expect("Failed to create db pool");
    run_migrations(pool.clone());
    pool
}

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

fn run_migrations(pool: Pool) {
    embedded_migrations::run(
        pool.get()
            .expect("No database connection from pool")
            .deref(),
    )
    .expect("Migrations could not be run");
}

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(connection) => Outcome::Success(DbConn(connection)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn serialize_base64<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(&value))
}

pub(crate) fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|string| base64::decode(&string).map_err(|err| Error::custom(err.to_string())))
}

pub(crate) fn deserialize_optional_base64<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).and_then(|optional_string| match optional_string {
        None => Ok(None),
        Some(content) => base64::decode(&content)
            .map(Some)
            .map_err(|err| Error::custom(err.to_string())),
    })
}

pub(crate) fn serialize_optional_base64<S>(
    maybe_value: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match maybe_value {
        None => serializer.serialize_none(),
        Some(value) => serialize_base64(value, serializer),
    }
}

pub type RepositoryQueryResult<T> = RepositoryResult<T, diesel::result::Error>;

pub enum RepositoryResult<T, E> {
    Ok(T),
    Err(RepositoryError<E>),
}

#[derive(Debug, Clone)]
pub enum RepositoryError<E> {
    Unauthorized(String),
    QueryError(E),
    NotFound,
}

impl<T> From<QueryResult<T>> for RepositoryQueryResult<T> {
    fn from(result: QueryResult<T>) -> Self {
        match result {
            Ok(value) => RepositoryQueryResult::Ok(value),
            Err(err) => RepositoryQueryResult::Err(RepositoryError::QueryError(err)),
        }
    }
}

impl<T> From<RepositoryQueryResult<T>> for Result<Json<T>, rocket::http::Status> {
    fn from(res: RepositoryQueryResult<T>) -> Self {
        match res {
            RepositoryResult::Ok(value) => Ok(Json(value)),
            RepositoryResult::Err(err) => match err {
                RepositoryError::Unauthorized(_) => Err(Status::Unauthorized),
                RepositoryError::QueryError(_) => Err(Status::InternalServerError),
                RepositoryError::NotFound => Err(Status::NotFound),
            },
        }
    }
}

pub fn to_status_created<T>(
    id: Uuid,
    path_with_leading_and_trailing_slash: &'static str,
    created_entry: Json<T>,
) -> status::Created<Json<T>> {
    let path = path_with_leading_and_trailing_slash;
    let host = env::var("ROCKET_ADDRESS").expect("ROCKET_ADDRESS must be set");
    let port = env::var("ROCKET_PORT").expect("ROCKET_PORT must be set");
    if !(path.starts_with('/') && path.ends_with('/')) {
        panic!("path for created entry is not valid");
    }
    status::Created(
        format!(
            "{host}:{port}{path}{id}",
            host = host,
            path = path,
            port = port,
            id = id,
        ),
        Some(created_entry),
    )
}
