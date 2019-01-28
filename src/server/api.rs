
use serde_json;
use std::str::from_utf8;
use server::error;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    username: String,
    email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    profile_picture: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiOkResponse {
    action: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    publish_to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    subscribe_to: Vec<String>,
    data: serde_json::Value,
}

impl ApiOkResponse {
    pub fn get_action_name(&self) -> String {
        self.action.to_owned()
    }
    pub fn get_data(&self) -> serde_json::Value {
        self.data.to_owned()
    }

    pub fn get_channels_to_subscribe_to(&self) -> Vec<String> {
        self.subscribe_to.to_owned()
    }

    pub fn get_channels_to_publish_to(&self) -> Vec<String> {
        self.publish_to.to_owned()
    }

    pub fn get_action(&self) -> String {
        self.action.to_owned()
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
}

impl ApiErrorResponse {
    pub fn get_error(self) -> String {
        self.error
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    channel_name: String,
}

impl Channel {
    pub fn new(channel_name: &str) -> Self {
        Self {
            channel_name: channel_name.to_owned()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResult {
    Ok(ApiOkResponse),
    Err(ApiErrorResponse)
}

impl ApiResult {
    pub fn parse_result(raw_bytes: &[u8]) -> Result<ApiResult, error::Error> {
        from_utf8(raw_bytes)
            .or_else(|err|
                Err(error::Error::ServerGarbageResponse(err.to_string()))
            )
            .and_then(|raw_string| {
                let api_result: ApiResult = serde_json::from_str(raw_string)
                    .or_else(|err| Err(error::Error::ServerSerialization(err.to_string())))?;

                Ok(api_result)
            })
    }
}

pub struct Api;

pub trait GetEndpoint {
    fn get_endpoint() -> String;
}

impl GetEndpoint for Api {
    fn get_endpoint() -> String {
        "https://866bc5bf-bee9-4ce6-b138-58c356e1cd00.mock.pstmn.io".to_string()
    }
}