use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder};

use crate::endpoints::summoner::SummonerEndpoint;
use crate::models::lcu_process::LcuProcess;
use crate::models::lockfile::Lockfile;

pub mod endpoints;
pub mod errors;
pub mod models;

pub type Result<T> = std::result::Result<T, errors::LcuDriverError>;

#[derive(Debug)]
pub struct LcuDriver {
    lcu_process: LcuProcess,
    lockfile: Lockfile,
    client: Client,
    base_url: String,
}

impl LcuDriver {
    pub fn new(lcu_process: LcuProcess, lockfile: Lockfile) -> Result<Self> {
        let mut headers = HeaderMap::with_capacity(2);
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Basic {}", lockfile.token))?,
        );

        let client = ClientBuilder::new()
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()?;

        let base_url = format!("https://127.0.0.1:{}", lockfile.port);

        Ok(LcuDriver {
            lcu_process,
            lockfile,
            client,
            base_url,
        })
    }

    fn format_url(&self, url: &str) -> String {
        format!("{}{}", self.base_url, url)
    }

    pub async fn get_current_summoner(&self) -> Result<String> {
        let summoner = self
            .client
            .get(self.format_url(&SummonerEndpoint::Current.url()))
            .send()
            .await?
            .text()
            .await?;

        Ok(summoner)
    }
}
