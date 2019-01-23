
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

use actix_web::middleware::identity::RequestIdentity;

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
            .and_then(|body| from_utf8(&body)
                .or_else(|err|
                    Err(err.to_string())
                )
                .and_then(move |raw| {
                    let v: ApiResult = serde_json::from_str(raw)
                        .or_else(|err| Err(err.to_string()))?;
                    match v {
                        ApiResult::Ok(res) => {
                            //let token = create_token(&user)?;
                            req.remember("token".to_string());
                            Ok(HttpResponse::Ok().body("hello world"))
                        },
                        ApiResult::Err(err) => Ok(HttpResponse::Unauthorized().json(json!({ "error": err.get_error() }))),
                    }

                })
                .or_else(|error_msg|
                    Ok(HttpResponse::BadGateway()
                        .json(json!({ "error": error_msg })))
                )
            )
        )
        .responder()
}

pub fn logout(req: &HttpRequest<AppState>) -> String {
    "test data".to_string()
}