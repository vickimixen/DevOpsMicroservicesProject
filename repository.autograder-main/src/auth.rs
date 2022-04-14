use jsonwebtoken as jwt;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, State};
use uuid::Uuid;

use crate::config;
use crate::config::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    #[serde(rename(serialize = "sub", deserialize = "sub"))]
    pub user_id: Uuid,
    pub is_superuser: bool,
    pub is_teacher: bool,
    pub is_student: bool,
    pub email: String,
    pub exp: u32,
}

impl Auth {
    pub fn token(&self, encoding_key: &EncodingKey) -> String {
        jwt::encode(&jwt::Header::new(Algorithm::RS256), self, encoding_key)
            .expect("not able to encode jwt")
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Auth, Self::Error> {
        let state: State<AppState> = request.guard()?;
        let decoding_key = DecodingKey::from_rsa_pem(&state.decoding_key)
            .unwrap_or_else(|e| panic!("couldn't generate public key: {}", e.to_string()));
        if let Some(auth) = extract_auth_from_request(request, &decoding_key) {
            Outcome::Success(auth)
        } else {
            Outcome::Failure((Status::Forbidden, ()))
        }
    }
}

fn extract_auth_from_request(request: &Request, decoding_key: &DecodingKey) -> Option<Auth> {
    request
        .headers()
        .get_one("authorization")
        .and_then(extract_token_from_header)
        .and_then(|token| decode_token(token, decoding_key))
}

fn extract_token_from_header(header: &str) -> Option<&str> {
    if let Some(stripped) = header.strip_prefix(config::TOKEN_PREFIX) {
        Some(stripped)
    } else {
        None
    }
}

/// Decode token into `Auth` struct. If any error is encountered, log it
/// and return None.
fn decode_token(token: &str, decoding_key: &DecodingKey) -> Option<Auth> {
    jwt::decode(token, decoding_key, &Validation::new(Algorithm::RS256))
        .map_err(|err| {
            error!("Auth decode error: {:?}", err);
        })
        .ok()
        .map(|token_data| token_data.claims)
}
