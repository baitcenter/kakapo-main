
use actix::prelude::*;

use actix_web::{
    App, AsyncResponder, Error,
    dev::JsonConfig, http, http::NormalizePath, HttpMessage,
    middleware, HttpRequest, HttpResponse,
    fs, fs::{NamedFile},
    FutureResponse, ResponseError, State,
};

use actix_web::middleware::cors::Cors;
use actix_web::middleware::Logger;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use chrono::Duration;

use serde_json;

use std::result::Result;
use std::result::Result::Ok;
use std::path::Path as fsPath;

use server::environment::Env;
use server::state::AppState;
use actix_web::Path;
use actix_web::Responder;

use futures::Future;
use actix_web::client;

use server::api::Api;
use server::api::GetEndpoint;
use actix_web::Json;

use std::str::from_utf8;
use server::api::ApiResult;

//se actix_web::middleware::identity::RequestIdentity;

type AsyncResponse = FutureResponse<HttpResponse>;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthData {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
struct SendingAuthData {
    user_identifier: String,
    password: String,
}

impl AuthData {
    pub fn into_sendable(self) -> SendingAuthData {
        SendingAuthData {
            user_identifier: self.username,
            password: self.password,
        }
    }
}

fn generate_tokens_from_auth_result(raw_bytes: &[u8]) -> HttpResponse {

    ApiResult::parse_result(raw_bytes)
        .and_then(|api_result| {
            match api_result {
                ApiResult::Ok(res) => {
                    //let token = create_token(&user)?;
                    let tokens = json!({
                                "token_type": "bearer",
                                "access_token": "JWT TOKEN",
                                "expires_in": "token_expiry",
                                "refresh_token": "generated_token"
                            });
                    Ok(HttpResponse::Ok().json(tokens))
                },
                ApiResult::Err(err) => Ok(HttpResponse::Unauthorized().json(json!({ "error": err.get_error() }))),
            }
        })
        .unwrap_or_else(|error_msg|
            HttpResponse::BadGateway()
                .json(json!({ "error": error_msg }))
        )
}

pub fn login((req, auth_data): (HttpRequest<AppState>, Json<AuthData>)) -> AsyncResponse {
    let endpoint = Api::get_endpoint();
    let login_endpoint = format!("{}users/authenticate", endpoint);

    client::ClientRequest::post(login_endpoint)
        .json(auth_data.into_inner().into_sendable())
        .unwrap_or_default()
        .send()
        .map_err(Error::from)
        .and_then(|resp| resp
            .body()
            .from_err()
            .and_then(|body| Ok(generate_tokens_from_auth_result(&body)))
        )
        .responder()
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshData {
    refresh_token: String,
}

pub fn refresh((req, auth_data): (HttpRequest<AppState>, Json<RefreshData>)) -> String {
    let state = req.state();
    debug!("state: {:?}", &state);
    "all_good".to_string()
}

pub fn logout(req: &HttpRequest<AppState>) -> String {
    "test data".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_token_generation_and_parsing() {

    }
}