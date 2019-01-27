use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use rand;

use std::collections::HashMap;
use std::mem;
use server::api::ApiOkResponse;
use server::api::ApiErrorResponse;
use server::api::Channel;

use server::sockets::Notification;

use uuid::Uuid;

type Client = Recipient<Notification>;
type Room = HashMap<usize, Client>;
#[derive(Default)]
pub struct WsServer {
    rooms: HashMap<String, Room>,
}

impl Actor for WsServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_async::<SendMsg>(ctx);
        self.subscribe_async::<SendErrorMsg>(ctx);
        self.subscribe_async::<JoinChannel>(ctx);
        self.subscribe_async::<LeaveChannel>(ctx);
    }
}

impl SystemService for WsServer {}
impl Supervised for WsServer {}

#[derive(Clone, Message)]
pub struct JoinChannel {
    id: Uuid,
    channel: String,
    client: Client,
}

impl JoinChannel {
    pub fn new(id: Uuid, channel: String, client: Client) -> Self {
        Self { id, channel, client }
    }
}

impl Handler<JoinChannel> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: JoinChannel, _ctx: &mut Self::Context) -> Self::Result {
        debug!("received request for joining channel");
    }
}

#[derive(Clone, Message)]
pub struct LeaveChannel {
    id: Uuid,
    channel: String,
}

impl LeaveChannel {
    pub fn new(id: Uuid, channel: String) -> Self {
        Self { id, channel }
    }
}

impl Handler<LeaveChannel> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveChannel, _ctx: &mut Self::Context) -> Self::Result {
        debug!("received request for leaving channel");
    }
}

#[derive(Clone, Message)]
pub struct SendMsg {
    id: Uuid,
    api_result: ApiOkResponse,
    client: Client,
}

impl SendMsg {
    pub fn new(id: Uuid, api_result: ApiOkResponse, client: Client) -> Self {
        Self { id, api_result, client }
    }
}

impl Handler<SendMsg> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SendMsg, _ctx: &mut Self::Context) -> Self::Result {
        debug!("handling message");
    }
}

#[derive(Clone, Message)]
pub struct SendErrorMsg {
    id: Uuid,
    error_result: ApiErrorResponse,
    client: Client,
}

impl SendErrorMsg {
    pub fn new(id: Uuid, error_result: ApiErrorResponse, client: Client) -> Self {
        Self { id, error_result, client }
    }
}

impl Handler<SendErrorMsg> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SendErrorMsg, _ctx: &mut Self::Context) -> Self::Result {
        debug!("handling error message");
    }
}

