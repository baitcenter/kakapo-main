
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct OkResponse {
    action: String,
    channels: Vec<serde_json::Value>,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    error: String,
    message: Option<String>,
}

impl ErrorResponse {
    pub fn get_error(self) -> String {
        self.error
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResult {
    Ok(OkResponse),
    Err(ErrorResponse)
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