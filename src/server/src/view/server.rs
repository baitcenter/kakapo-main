
use actix::prelude::*;

use actix_web::{
    App, Error as ActixError,
    dev::JsonConfig, error as http_error, http, http::NormalizePath, middleware,
    HttpRequest, HttpResponse, fs, fs::{NamedFile},
    ResponseError, State,
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

use connection;
use connection::executor::DatabaseExecutor;

// current module
use view::procedure;
use model::actions;

use super::state::AppState;
use super::procedure::{ ProcedureBuilder, NoQuery };
use super::extensions::CorsBuilderProcedureExt;

use view::error;

use std::error::Error;
use data;
use view::environment::Env;
use view::action_wrapper::Broadcaster;

//static routes
fn index(_state: State<AppState>) -> Result<NamedFile, ActixError> {
    let www_path = Env::www_path();
    let path = fsPath::new(&www_path).join("index.html");
    Ok(NamedFile::open(path)?)
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAllEntities {
    #[serde(default)]
    pub show_deleted: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntity {
    pub name: String,
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SocketRequest {
    GetTables { show_deleted: bool },
    StopGetTables,
}


#[derive(Clone)]
struct SessionHandler {}

impl SessionHandler {
    pub fn new() -> Self {
        Self {}
    }
}


pub fn serve() {

    let connection = connection::executor::Connector::new()
        .host(Env::database_host())
        .port(Env::database_port())
        .user(Env::database_user())
        .pass(Env::database_pass())
        .db(Env::database_db())
        .done();

    let server_addr = Env::server_addr();
    let is_secure = Env::is_secure();

    let mut server_cfg = actix_web::server::new(move || {

        let www_path = Env::www_path();
        let script_path = Env::script_path();
        let state = AppState::new(connection.clone(), &script_path, "Kakapo");

        App::with_state(state)
            .middleware(Logger::new("Responded [%s] %b bytes %Dms"))
            .middleware(Logger::new("Requested [%r] FROM %a \"%{User-Agent}i\""))
            .middleware(IdentityService::new(
                CookieIdentityPolicy::new(Env::secret_key().as_bytes())
                    .name("kakapo-server")
                    .path("/")
                    .domain(Env::domain())
                    .max_age(Duration::days(1))
                    .secure(is_secure), // this can only be true if you have https
            ))
            .configure(|app| Cors::for_app(app)
                //.allowed_origin("http://localhost:3000") //TODO: this doesn't work in the current version of cors middleware https://github.com/actix/actix-web/issues/603
                //.allowed_origin("http://localhost:8080")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .procedure(
                    "/manage/getAllTables",
                    |_: NoQuery, get_all_entities: GetAllEntities|
                        actions::GetAllEntities::<data::Table, Broadcaster>::new(get_all_entities.show_deleted)
                )
                .procedure(
                    "/manage/getAllQueries",
                    |_: NoQuery, get_all_entities: GetAllEntities|
                        actions::GetAllEntities::<data::Query, Broadcaster>::new(get_all_entities.show_deleted)
                )
                .procedure(
                    "/manage/getAllScripts",
                    |_: NoQuery, get_all_entities: GetAllEntities|
                        actions::GetAllEntities::<data::Script, Broadcaster>::new(get_all_entities.show_deleted)
                )

                .procedure(
                    "/manage/getTable",
                    |_: NoQuery, get_entity: GetEntity|
                        actions::GetEntity::<data::Table, Broadcaster>::new(get_entity.name)
                )
                .procedure(
                    "/manage/getQuery",
                    |_: NoQuery, get_entity: GetEntity|
                        actions::GetEntity::<data::Query, Broadcaster>::new(get_entity.name)
                )
                .procedure(
                    "/manage/getScript",
                    |_: NoQuery, get_entity: GetEntity|
                        actions::GetEntity::<data::Script, Broadcaster>::new(get_entity.name)
                )
                .procedure(
                    "/manage/createTable",
                    |entity: data::Table, _: NoQuery|
                        actions::CreateEntity::<data::Table, Broadcaster>::new(entity)
                )
                .procedure(
                    "/manage/createQuery",
                    |entity: data::Query, _: NoQuery|
                        actions::CreateEntity::<data::Query, Broadcaster>::new(entity)
                )
                .procedure(
                    "/manage/createScript",
                    |entity: data::Script, _: NoQuery|
                        actions::CreateEntity::<data::Script, Broadcaster>::new(entity)
                )
                .procedure(
                    "/manage/updateTable",
                    |entity: data::Table, get_entity: GetEntity|
                        actions::UpdateEntity::<data::Table, Broadcaster>::new(get_entity.name, entity)
                )
                .procedure(
                    "/manage/updateQuery",
                    |entity: data::Query, get_entity: GetEntity|
                        actions::UpdateEntity::<data::Query, Broadcaster>::new(get_entity.name, entity)
                )
                .procedure(
                    "/manage/updateScript",
                    |entity: data::Script, get_entity: GetEntity|
                        actions::UpdateEntity::<data::Script, Broadcaster>::new(get_entity.name, entity)
                )
                .procedure(
                    "/manage/deleteTable",
                    |_: NoQuery, get_entity: GetEntity|
                        actions::DeleteEntity::<data::Table, Broadcaster>::new(get_entity.name)
                )
                .procedure(
                    "/manage/deleteQuery",
                    |_: NoQuery, get_entity: GetEntity|
                        actions::DeleteEntity::<data::Query, Broadcaster>::new(get_entity.name)
                )
                .procedure(
                    "/manage/deleteScript",
                    |_: NoQuery, get_entity: GetEntity|
                        actions::DeleteEntity::<data::Script, Broadcaster>::new(get_entity.name)
                )
                .procedure(
                    "/manage/queryTableData",
                    |_: NoQuery, get_table: GetEntity|
                        actions::QueryTableData::<Broadcaster>::new(get_table.name)
                )
                .procedure(
                    "/manage/insertTableData",
                    |data: data::TableData, get_table: GetEntity|
                        actions::InsertTableData::<Broadcaster>::new(get_table.name, data)
                )
                .procedure(
                    "/manage/updateTableData",
                    |keyed_data: data::KeyedTableData, get_table: GetEntity|
                        actions::UpdateTableData::<Broadcaster>::new(get_table.name, keyed_data)
                )
                .procedure(
                    "/manage/deleteTableData",
                    |keys: data::KeyData, get_table: GetEntity|
                        actions::DeleteTableData::<Broadcaster>::new(get_table.name, keys)
                )
                .procedure(
                    "/manage/runQuery",
                    |params: data::QueryParams, get_query: GetEntity|
                        actions::RunQuery::<Broadcaster>::new(get_query.name, params)
                )
                .procedure(
                    "/manage/runScript",
                    |param: data::ScriptParam, get_script: GetEntity|
                        actions::RunScript::<Broadcaster>::new(get_script.name, param)
                )
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