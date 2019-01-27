
mod environment;
mod state;
mod api;
mod auth_routes;
mod sockets;
mod socket_server;
mod error;

use actix::prelude::*;

use actix_web::{
    App, AsyncResponder, Error, dev::JsonConfig,
    http, http::NormalizePath, http::Method,
    HttpMessage, middleware, HttpRequest, HttpResponse,
    fs, fs::NamedFile,
    ResponseError, State, ws,
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

//static routes
fn index(_state: State<AppState>) -> Result<NamedFile, Error> {
    let www_path = Env::www_path();
    let path = fsPath::new(&www_path).join("index.html");
    Ok(NamedFile::open(path)?)
}

type AsyncResponse = Box<Future<Item=HttpResponse, Error=Error>>;

pub struct ActionRequest {
    action: String,
    params: serde_json::Value,
    data: serde_json::Value,
}

fn call_internal_api(req: &HttpRequest<AppState>) -> AsyncResponse {
    client::ClientRequest::get("http://icanhazip.com/")
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|resp| {
            resp
            .body()
            .from_err()
            .and_then(|body| {
                Ok(HttpResponse::Ok().body(body))
            })
        })
        .responder()
}

pub fn serve() {

    let server_addr = Env::server_addr();
    let is_secure = Env::is_secure();

    let mut server_cfg = actix_web::server::new(move || {

        let www_path = Env::www_path();
        let secret = Env::secret_key();
        let state = AppState::new("KakapoArbiter", &secret);

        App::with_state(state)
            .middleware(Logger::new("Responded [%s] %b bytes %Dms"))
            .middleware(Logger::new("Requested [%r] FROM %a \"%{User-Agent}i\""))
            .configure(|app| Cors::for_app(app)
                //.allowed_origin("http://localhost:3000") //TODO: this doesn't work in the current version of cors middleware https://github.com/actix/actix-web/issues/603
                //.allowed_origin("http://localhost:8080")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/listen", |r| r.f(sockets::handler))
                .resource("/login", |r| r.method(Method::POST).with(auth_routes::login))
                .resource("/logout", |r| r.f(auth_routes::logout))
                .resource("/manage/{param}", |r| r.f(call_internal_api))
                .register())
            .resource("/", |r| {
                r.method(http::Method::GET).with(index)
            })
            .default_resource(|r| r.h(NormalizePath::default()))
            .handler(
                "/",
                fs::StaticFiles::new(fsPath::new(&www_path))
                    .unwrap()
                    .show_files_listing())
    });

    server_cfg = server_cfg
        .workers(num_cpus::get())
        .keep_alive(30);

    debug!("is_secure: {:?}", is_secure);
    let http_server = if is_secure {
        let ssl_cert_privkey_path = Env::ssl_cert_privkey_path();
        let ssl_cert_fullchain_path = Env::ssl_cert_fullchain_path();

        let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        ssl_builder
            .set_private_key_file(ssl_cert_privkey_path, SslFiletype::PEM)
            .unwrap();
        ssl_builder.set_certificate_chain_file(ssl_cert_fullchain_path).unwrap();


        server_cfg
            .bind_ssl(&server_addr, ssl_builder)
            .unwrap()

    } else {
        server_cfg
            .bind(&server_addr)
            .unwrap()
    };

    http_server
        .shutdown_timeout(30)
        .start();

    info!("Kakapo server started on \"{}\"", &server_addr);

}