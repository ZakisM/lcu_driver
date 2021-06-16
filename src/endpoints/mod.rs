use reqwest::header::HeaderMap;
use reqwest::Method;

pub mod champ_select;
pub mod gameflow;
pub mod perks;
pub mod summoner;

pub struct EndpointInfo {
    pub url: String,
    pub method: Method,
    pub headers: Option<HeaderMap>,
    pub body: Option<String>,
}
