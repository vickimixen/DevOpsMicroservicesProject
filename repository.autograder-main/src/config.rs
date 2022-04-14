use std::env;

use jsonwebtoken::EncodingKey;
use rocket::fairing::AdHoc;
use rocket::Rocket;

const PRIVATE_KEY_ENV: &str = "CORE_PRIVATE_KEY";
const PUBLIC_KEY_ENV: &str = "CORE_PUBLIC_KEY";
pub const TOKEN_PREFIX: &str = "Bearer ";

pub struct AppState {
    pub encoding_key: EncodingKey,
    pub decoding_key: Vec<u8>,
}

impl AppState {
    pub fn manage() -> AdHoc {
        AdHoc::on_attach("Manage config", |rocket: Rocket| {
            let private_key = env::var(PRIVATE_KEY_ENV)
                .map(|key| {
                    EncodingKey::from_rsa_pem(&key.into_bytes()).unwrap_or_else(|err| {
                        panic!("private key could not be read: {}", err.to_string())
                    })
                })
                .unwrap_or_else(|_| panic!("No {} environment variable found", PRIVATE_KEY_ENV));
            Ok(rocket.manage(AppState {
                encoding_key: private_key,
                decoding_key: env::var(PUBLIC_KEY_ENV)
                    .map(|key| key.into_bytes())
                    .unwrap_or_else(|_| panic!("No {} environment variable found", PUBLIC_KEY_ENV)),
            }))
        })
    }
}
