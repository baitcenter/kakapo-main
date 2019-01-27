
use serde_json;
use std::str::from_utf8;
use server::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiOkResponse {
    action: String,
    channels: Vec<serde_json::Value>,
    data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    error: String,
    message: Option<String>,
}

impl ApiErrorResponse {
    pub fn get_error(self) -> String {
        self.error
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        "http://localhost:8001/".to_string()
    }
}