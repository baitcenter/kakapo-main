
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

use state::AppState;
use state::JwtConfig;
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
use chrono::Utc;

use jsonwebtoken as jwt;
use server::error::Error as ServerError;

type AsyncResponse = FutureResponse<HttpResponse>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthData {
    username: String,
    password: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Claims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
    is_admin: bool,
    roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshData {
    refresh_token: String,
}

impl Claims {
    pub fn new<S>(username: String, state: &S) -> Self
        where S: JwtConfig,
    {
        let duration = state.get_jwt_duration();
        Self {
            iss: state.get_jwt_issuer(),
            sub: username,
            iat: Utc::now().timestamp(),
            exp: (Utc::now() + Duration::seconds(duration)).timestamp(),
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

fn create_token<S>(state: &S, claims: Claims, session_token: String) -> Result<serde_json::Value, ServerError>
    where S: JwtConfig,
{
    let secret = state.get_secret_key();
    let expires_in = state.get_jwt_duration();

    let jwt_token = jwt::encode(&jwt::Header::default(), &claims, secret.as_ref())
        .or_else(|err| Err(ServerError::EncodeError))?;

    Ok(json!({
        "token_type": "bearer",
        "access_token": jwt_token,
        "expires_in": expires_in,
        "refresh_token": session_token
    }))
}

fn generate_jwt_token_from_auth_response<S>(state: &S, auth_data: AuthenticationResponse) -> Result<serde_json::Value, ServerError>
    where S: JwtConfig,
{
    let claims = Claims::new(auth_data.username, state)
        .with_set_admin(auth_data.is_admin)
        .with_set_roles(auth_data.roles)
        .to_owned();

    if claims.exp >= auth_data.session_token_exp.timestamp() {
        error!("Cannot have a jwt that expires after the access token");
        return Err(ServerError::ExpiryDateTooShort);
    }

    info!("generating jwt token for user: {:?}", &claims.sub);
    create_token(state, claims, auth_data.session_token)
}

fn generate_tokens_from_action_result<S>(state: &S, api_result: ApiOkResponse) -> HttpResponse
    where S: JwtConfig,
{
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

fn generate_tokens_from_auth_result<S>(state: &AppState, raw_bytes: &[u8]) -> HttpResponse {

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

fn return_server_response(raw_bytes: &[u8]) -> HttpResponse {
    ApiResult::parse_result(raw_bytes)
        .and_then(|api_result| match api_result {
            ApiResult::Ok(res) => {
                Ok(HttpResponse::Ok().json(res.get_data()))
            },
            ApiResult::Err(err) => {
                warn!("Did not receive a valid response from the server");
                Ok(HttpResponse::BadRequest().json(json!({ "error": err.get_error().to_string() })))
            },
        })
        .unwrap_or_else(|error_msg|
            HttpResponse::BadGateway()
                .json(json!({ "error": error_msg.to_string() }))
        )
}

pub fn login((req, auth_data): (HttpRequest<AppState>, Json<AuthData>)) -> AsyncResponse {
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
            .and_then(move |body| Ok(generate_tokens_from_auth_result::<AppState>(req.state(), &body)))
        )
        .responder()
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
            .and_then(move |body| Ok(generate_tokens_from_auth_result::<AppState>(req.state(), &body)))
        )
        .responder()
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
    use std::collections::HashMap;
    use std::str;
    use actix_web::test::TestServer;
    use actix_web::Body;
    use actix_web::Binary;
    use actix_web::test::TestApp;
    use actix_web::dev::Resource;
    use actix_web::test::TestServerBuilder;


    struct TestState;
    impl JwtConfig for TestState {
        fn get_jwt_issuer(&self) -> String {
            "http://www.example.com/".to_string()
        }

        fn get_jwt_duration(&self) -> i64 {
            15 * 60 // 15 minutes
        }

        fn get_secret_key(&self) -> String {
            "hunter2".to_string()
        }
    }

    #[test]
    fn test_claims_generation() {
        let state = TestState;
        let username = "chunkylover53".to_string();

        let claims = Claims::new(username, &state)
            .with_set_roles(vec!["inspector".to_string()])
            .with_set_admin(true)
            .to_owned();

        assert_eq!(claims.iss, "http://www.example.com/");
        assert_eq!(claims.sub, "chunkylover53");
        assert_eq!(claims.exp - claims.iat, 15 * 60);
        assert_eq!(claims.roles, vec!["inspector"]);
        assert_eq!(claims.is_admin, true);
    }

    #[test]
    fn test_token_generation() {
        let state = TestState;
        let username = "chunkylover53".to_string();

        let claims = Claims::new(username, &state)
            .to_owned();

        let token = create_token(&state, claims.to_owned(), "my_session_token".to_string()).unwrap();
        let map: HashMap<String, serde_json::Value> = serde_json::from_value(token.to_owned()).unwrap();

        assert_eq!(token.get("token_type").unwrap().to_owned(), json!("bearer"));
        assert_eq!(token.get("expires_in").unwrap().to_owned(), json!(15 * 60));
        assert_eq!(token.get("refresh_token").unwrap().to_owned(), json!("my_session_token"));

        let access_token: String = serde_json::from_value(token.get("access_token").unwrap().to_owned()).unwrap();
        let final_claims: Claims = jwt::decode(&access_token, "hunter2".as_ref(), &jwt::Validation::default()).unwrap().claims;

        assert_eq!(final_claims, claims);
    }

    #[test]
    fn test_token_generation_from_auth_response() {
        let state = TestState;
        let auth_data = AuthenticationResponse {
            username: "chunkylover53".to_string(),
            is_admin: true,
            roles: vec!["inspector".to_string()],
            session_token: "my_session_token".to_string(),
            session_token_exp: (Utc::now() + Duration::days(1)).naive_utc(),
        };

        let token = generate_jwt_token_from_auth_response(&state, auth_data).unwrap();
        let map: HashMap<String, serde_json::Value> = serde_json::from_value(token.to_owned()).unwrap();
        let access_token: String = serde_json::from_value(token.get("access_token").unwrap().to_owned()).unwrap();
        let claims: Claims = jwt::decode(&access_token, "hunter2".as_ref(), &jwt::Validation::default()).unwrap().claims;

        assert_eq!(token.get("token_type").unwrap().to_owned(), json!("bearer"));
        assert_eq!(token.get("expires_in").unwrap().to_owned(), json!(15 * 60));
        assert_eq!(token.get("refresh_token").unwrap().to_owned(), json!("my_session_token"));

        assert_eq!(claims.sub, "chunkylover53".to_string());
        assert_eq!(claims.is_admin, true);
        assert_eq!(claims.roles, vec!["inspector"]);
    }

    #[test]
    fn test_token_generation_from_auth_response_if_exp_is_too_short() {
        let state = TestState;
        let auth_data = AuthenticationResponse {
            username: "chunkylover53".to_string(),
            is_admin: true,
            roles: vec!["inspector".to_string()],
            session_token: "my_session_token".to_string(),
            session_token_exp: (Utc::now() + Duration::minutes(5)).naive_utc(), //5 minutes < 15 minutes, this is bad
        };

        let error = generate_jwt_token_from_auth_response(&state, auth_data).unwrap_err();
        assert_eq!(error, ServerError::ExpiryDateTooShort);
    }

    #[test]
    fn test_generate_token_from_server_response() {
        let state = TestState;
        let tomorrow_date = (Utc::now() + Duration::days(1))
            .naive_utc().format("%Y-%m-%dT%H:%M:%S").to_string();
        let api_result_json = json!({
            "action": "AuthResult",
            "data": {
                "username": "chunkylover53",
                "isAdmin": true,
                "roles": ["7G", "inspector"],
                "sessionToken": "my_session_token",
                "sessionTokenExp": tomorrow_date,
            },
        });

        let api_result: ApiOkResponse = serde_json::from_value(api_result_json).unwrap();
        let response = generate_tokens_from_action_result(&state, api_result);

        assert_eq!(response.status(), http::StatusCode::OK);

    }

    #[test]
    fn test_generate_token_from_server_response_with_failed_auth() {
        let state = TestState;
        let api_result_json = json!({
            "action": "AuthResult",
            "data": null,
        });

        let api_result: ApiOkResponse = serde_json::from_value(api_result_json).unwrap();
        let response = generate_tokens_from_action_result(&state, api_result);

        assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_generate_token_from_server_response_with_bad_data() {
        let state = TestState;
        let api_result_json = json!({
            "action": "AuthResult",
            "data": {
                "bad": "data"
            },
        });

        let api_result: ApiOkResponse = serde_json::from_value(api_result_json).unwrap();
        let response = generate_tokens_from_action_result(&state, api_result);

        assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
    }



}