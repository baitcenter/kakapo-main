
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

use actix_broker::BrokerIssue;

use server::socket_server::WsServer;
use server::socket_server::JoinChannel;
use server::socket_server::LeaveChannel;
use server::socket_server::SendMsg;
use server::socket_server::SendErrorMsg;

use server::error;
use server::api::ApiResult;
use server::api::Channel;

use uuid::Uuid;


pub fn handler(req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    ws::start(req, WsSessionManager::new())
}

#[derive(Clone, Debug)]
struct WsSessionManager {
    id: Uuid,
}

impl WsSessionManager {
    fn new() -> Self {
        let id = Uuid::new_v4();
        Self { id }
    }
}

impl Actor for WsSessionManager {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("WsSession[{:?}] opened ", &self.id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WsSession[{:?}] closed ", &self.id);
    }
}

#[derive(Debug, Clone, Message)]
pub struct Notification {
    data: serde_json::Value,
}

impl Notification {
    pub fn get_data(self) -> serde_json::Value {
        self.data
    }
}

impl Handler<Notification> for WsSessionManager {
    type Result = ();

    fn handle(&mut self, notification: Notification, ctx: &mut Self::Context) {
        let data = notification.get_data();
        serde_json::to_string(&data)
            .and_then(|res| {
                ctx.text(res);
                Ok(())
            })
            .or_else(|err| {
                error!("Could not parse message for notifactions: {:?}", &err);
                Err(err)
            });

    }
}


#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "camelCase")]
enum WsInputData {
    Subscribe {
        channel: String,
    },
    Unsubscribe {
        channel: String,
    },
    Call {
        function: String,
        params: serde_json::Value,
        data: serde_json::Value,
    },
}

impl WsSessionManager {
    fn subscribe_to_channel(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self, AppState>,
        channel_name: String,
    ) {
        let join = JoinChannel::new(
            self.id,
            channel_name,
            ctx.address().recipient(),
        );

        WsServer::from_registry()
            .send(join.to_owned())
            .into_actor(self)
            .then(|res, act, _ctx| {
                info!("Got server from registry");
                fut::ok(())
            }).spawn(ctx);

        self.issue_sync(join, ctx);

    }

    fn unsubscribe_from_channel(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self, AppState>,
        channel_name: String,
    ) {
        let leave = LeaveChannel::new(
            self.id,
            channel_name,
        );

        self.issue_sync(leave, ctx);
    }

    fn process_procedure_result(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self, AppState>,
        raw_bytes: &[u8],
    ) -> Result<(), error::Error> {
        ApiResult::parse_result(raw_bytes)
            .and_then(|api_result| match api_result {
                ApiResult::Ok(res) => {
                    debug!("received ok message \"{:?}\"", &res);
                    let recipient = ctx.address().recipient();
                    let send_msg = SendMsg::new(self.id, res, recipient);
                    self.issue_sync(send_msg, ctx);
                    Ok(())
                },
                ApiResult::Err(err) => {
                    debug!("received err message \"{:?}\"", &err);
                    let recipient = ctx.address().recipient();
                    let send_msg = SendErrorMsg::new(self.id, err, recipient);
                    self.issue_sync(send_msg, ctx);
                    Ok(())
                },
            })
            .or_else(|err| {
                Err(err)
            })
    }

    fn call_procedure(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self, AppState>,
        function: String,
        params: serde_json::Value,
        data: serde_json::Value,
    ) {
        client::ClientRequest::post("https://example.com")
            .json(data)
            .unwrap_or_default()
            .send()
            .map_err(Error::from) //TODO: wait here?
            .and_then(|resp| resp
                .body()
                .from_err()
                .and_then(|body| {
                    self.process_procedure_result(ctx, &body)
                        .or_else(|err| {
                            debug!("encountered error: {:?}", &err);
                            Ok(()) //the error is handled in the process function
                        })
                })
            );
    }

    fn handle_message(&mut self, ctx: &mut ws::WebsocketContext<Self, AppState>, input: WsInputData) {
        match input {
            WsInputData::Subscribe { channel } => {
                self.subscribe_to_channel(ctx, channel);
            },
            WsInputData::Unsubscribe { channel } => {
                self.unsubscribe_from_channel(ctx, channel);
            },
            WsInputData::Call { function, params, data } => {
                self.call_procedure(ctx, function, params, data);
            },
        }
    }
}


impl StreamHandler<ws::Message, ws::ProtocolError> for WsSessionManager {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        debug!("received msg \"{:?}\"", msg);
        match msg {
            ws::Message::Text(text) => {
                serde_json::from_str(&text)
                    .or_else(|err| {
                        debug!("could not understand incoming message, must be `WsInputData`");
                        Err(())
                    })
                    .and_then(move |res: WsInputData| {
                        self.handle_message(ctx, res);
                        Ok(())
                    });

            },
            ws::Message::Close(_) => {
                ctx.stop();
            },
            ws::Message::Binary(_) => {
                info!("binary websocket messages not currently supported");
            },
            ws::Message::Ping(_) => {

            },
            ws::Message::Pong(_) => {

            },
        }
    }
}
