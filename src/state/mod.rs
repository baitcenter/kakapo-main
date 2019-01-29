

use actix::prelude::*;

pub mod api;
pub mod error;

#[derive(Debug, Clone)]
pub struct AppState {
    app_name: String,
    secret_key: String,
}


const JWT_TIMEOUT:i64 = 5 * 60;

impl AppState {
    pub fn new(app_name: &str, secret: &str) -> Self {
        AppState {
            app_name: app_name.to_string(),
            secret_key: secret.to_string(),
        }
    }

    pub fn get_jwt_issuer(&self) -> String {
        "http://localhost:8000".to_string() //TODO:...
    }

    pub fn get_jwt_duration(&self) -> i64 {
        JWT_TIMEOUT
    }

    pub fn get_secret_key(&self) -> String {
        self.secret_key.to_owned()
    }
}
