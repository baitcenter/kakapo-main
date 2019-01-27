
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
use server::api::ApiOkResponse;

use chrono::NaiveDateTime;
use chrono::Local;

use jsonwebtoken as jwt;

//se actix_web::middleware::identity::RequestIdentity;

type AsyncResponse = FutureResponse<HttpResponse>;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthData {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthenticationResponse {
    username: String,
    is_admin: bool,
    roles: Vec<String>,
    session_token: String,
    session_token_exp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
    is_admin: bool,
    roles: Vec<String>,

}

impl Claims {
    pub fn new(username: String, state: &AppState) -> Self {
        let duration = state.get_jwt_duration();
        Self {
            iss: state.get_jwt_issuer(),
            sub: username,
            iat: Local::now().timestamp(),
            exp: (Local::now() + Duration::seconds(duration)).timestamp(),
            is_admin: false,
            roles: vec![],
        }
    }

    pub fn with_set_admin(&mut self, is_admin: bool) -> &mut Self {
        self.is_admin = is_admin;
        self
    }

    pub fn with_set_roles(&mut self, roles: Vec<String>) -> &mut Self {
        self.roles = roles;
        self
    }
}

fn create_token(state: &AppState, claims: Claims, session_token: String) -> Result<serde_json::Value, String> {
    let secret = state.get_secret_key();
    let expires_in = state.get_jwt_duration();

    let jwt_token = jwt::encode(&jwt::Header::default(), &claims, secret.as_ref())
        .or_else(|err| Err("Could not decode jwt token".to_string()))?;

    Ok(json!({
        "token_type": "bearer",
        "access_token": jwt_token,
        "expires_in": expires_in,
        "refresh_token": session_token
    }))
}

fn generate_jwt_token_from_auth_response(state: &AppState, auth_data: AuthenticationResponse) -> Result<serde_json::Value, String> {
    let claims = Claims::new(auth_data.username, state)
        .with_set_admin(auth_data.is_admin)
        .with_set_roles(auth_data.roles)
        .to_owned();

    if claims.exp >= auth_data.session_token_exp.timestamp() {
        error!("Cannot have a jwt that expires after the access token");
        return Err("invalid expiry for the session token".to_string());
    }

    info!("generating jwt token for user: {:?}", &claims.sub);
    create_token(state, claims, auth_data.session_token)
}

fn generate_tokens_from_action_result(state: &AppState, api_result: ApiOkResponse) -> HttpResponse {
    let action_name = api_result.get_action_name();
    if action_name != "AuthResult" {
        warn!("Expected AuthResult to be returned from result");
        return HttpResponse::InternalServerError()
            .json(json!({ "error": "Assertion failed" }))
    }

    let auth_response: Option<AuthenticationResponse> =
        serde_json::from_value(api_result.get_data())
            .unwrap_or_else(|res| {
                warn!("Could not deserialize the auth result from the server");
                None
            });

    if let Some(auth_data) = auth_response {
        generate_jwt_token_from_auth_response(state, auth_data)
            .and_then(|token| {
                Ok(HttpResponse::Ok().json(token))
            })
            .unwrap_or_else(|err_msg| {
                HttpResponse::InternalServerError()
                    .json(json!({ "error": err_msg }))
            })
    } else {
        info!("could not authorize user");
        HttpResponse::Unauthorized().json(json!({ "error": "not authorized" }))
    }
}

fn generate_tokens_from_auth_result(state: &AppState, raw_bytes: &[u8]) -> HttpResponse {

    ApiResult::parse_result(raw_bytes)
        .and_then(|api_result| match api_result {
            ApiResult::Ok(res) => {
                Ok(generate_tokens_from_action_result(state, res))
            },
            ApiResult::Err(err) => {
                warn!("Did not receive a valid response from the server");
                Ok(HttpResponse::Unauthorized().json(json!({ "error": err.get_error() })))
            },
        })
        .unwrap_or_else(|error_msg|
            HttpResponse::BadGateway()
                .json(json!({ "error": error_msg }))
        )
}

pub fn login((req, auth_data): (HttpRequest<AppState>, Json<AuthData>)) -> AsyncResponse {
    let endpoint = Api::get_endpoint();
    let login_endpoint = format!("{}/users/authenticate", endpoint);

    client::ClientRequest::post(login_endpoint)
        .json(auth_data.into_inner().into_sendable())
        .unwrap_or_default()
        .send()
        .map_err(Error::from)
        .and_then(|resp| resp
            .body()
            .from_err()
            .and_then(move |body| Ok(generate_tokens_from_auth_result(req.state(), &body)))
        )
        .responder()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshData {
    refresh_token: String,
}

pub fn refresh((req, auth_data): (HttpRequest<AppState>, Json<RefreshData>)) -> AsyncResponse {
    let endpoint = Api::get_endpoint();
    let login_endpoint = format!("{}/users/authenticate", endpoint);

    client::ClientRequest::post(login_endpoint)
        .json(auth_data.into_inner())
        .unwrap_or_default()
        .send()
        .map_err(Error::from)
        .and_then(|resp| resp
            .body()
            .from_err()
            .and_then(move |body| Ok(generate_tokens_from_auth_result(req.state(), &body)))
        )
        .responder()
}

fn return_server_response(raw_bytes: &[u8]) -> HttpResponse {
    ApiResult::parse_result(raw_bytes)
        .and_then(|api_result| match api_result {
            ApiResult::Ok(res) => {
                Ok(HttpResponse::Ok().json(res.get_data()))
            },
            ApiResult::Err(err) => {
                warn!("Did not receive a valid response from the server");
                Ok(HttpResponse::BadRequest().json(json!({ "error": err.get_error() })))
            },
        })
        .unwrap_or_else(|error_msg|
            HttpResponse::BadGateway()
                .json(json!({ "error": error_msg }))
        )
}

pub fn logout((req, auth_data): (HttpRequest<AppState>, Json<RefreshData>)) -> AsyncResponse {
    let endpoint = Api::get_endpoint();
    let logout_endpoint = format!("{}/users/remove_authentication", endpoint);

    client::ClientRequest::post(logout_endpoint)
        .json(auth_data.into_inner())
        .unwrap_or_default()
        .send()
        .map_err(Error::from)
        .and_then(|resp| resp
            .body()
            .from_err()
            .and_then(move |body| Ok(return_server_response(&body)))
        )
        .responder()
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_token_generation_and_parsing() {

    }
}