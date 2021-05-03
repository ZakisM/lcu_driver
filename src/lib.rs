use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Client, ClientBuilder, Method, Request};
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

use crate::endpoints::champ_select::{ChampSelectEndpoint, ChampSelectSession};
use crate::endpoints::perks::{PerksEndpoint, PerksPage};
use crate::endpoints::summoner::{Summoner, SummonerEndpoint};
use crate::endpoints::EndpointInfo;
use crate::errors::LcuDriverError;
use crate::models::lcu_process::LcuProcess;
use crate::models::lockfile::Lockfile;

pub mod endpoints;
pub mod errors;
pub mod models;

pub type Result<T> = std::result::Result<T, errors::LcuDriverError>;

// Possible states for the driver
#[derive(Debug)]
pub struct Uninitialized {}

#[derive(Debug)]
pub struct Initialized {}

#[derive(Debug)]
struct LcuDriverInner {
    lcu_process: LcuProcess,
    lockfile: Lockfile,
    client: Client,
    api_base_url: String,
    league_install_dir: PathBuf,
}

impl LcuDriverInner {
    async fn replace_inner(&mut self, new_lcu_driver: Arc<LcuDriver<Initialized>>) {
        let new_inner = new_lcu_driver.inner.read().await;

        self.lcu_process = new_inner.lcu_process.clone();
        self.lockfile = new_inner.lockfile.clone();
        self.client = new_inner.client.clone();
        self.api_base_url = new_inner.api_base_url.clone();
        self.league_install_dir = new_inner.league_install_dir.clone();
    }
}

#[derive(Debug)]
pub struct LcuDriver<S> {
    inner: RwLock<LcuDriverInner>,
    _state: S,
}

impl LcuDriver<Uninitialized> {
    pub async fn connect() -> Result<LcuDriver<Initialized>> {
        let lcu_process = LcuProcess::locate().await?;

        let league_install_dir = Path::new(
            lcu_process
                .get_argument_value("install-directory=")
                .ok_or_else(|| LcuDriverError::new("Failed to find League install directory"))?,
        )
        .to_path_buf();

        let lockfile = Lockfile::load(league_install_dir.join("lockfile")).await?;

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

        let api_base_url = format!("https://127.0.0.1:{}", lockfile.port);

        let inner_instance = LcuDriverInner {
            lcu_process,
            lockfile,
            client,
            api_base_url,
            league_install_dir,
        };

        Ok(LcuDriver {
            inner: RwLock::new(inner_instance),
            _state: Initialized {},
        })
    }

    async fn connect_wait_lockfile(watch_lockfile: bool) -> Arc<LcuDriver<Initialized>> {
        loop {
            if let Ok(lcu_driver) = LcuDriver::connect().await {
                //Check that we can actually connect to the client
                if lcu_driver.get_current_summoner().await.is_ok() {
                    let pointer = Arc::new(lcu_driver);

                    if watch_lockfile {
                        LcuDriver::start_lockfile_watching(pointer.clone());
                    }

                    return pointer;
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn connect_wait() -> Arc<LcuDriver<Initialized>> {
        LcuDriver::connect_wait_lockfile(true).await
    }
}

impl LcuDriver<Initialized> {
    /*
        Spawn a task that will check if the lockfile has changed. If the lockfile exists but has
        new data, then it will be updated. However, if the lockfile no longer exists then any
        calls made from the driver will block until the process is detected and the lockfile
        exists.
    */
    fn start_lockfile_watching(lcu_driver: Arc<LcuDriver<Initialized>>) {
        tokio::task::spawn(async move {
            loop {
                let inner = lcu_driver.inner.read().await;

                if !inner.lockfile.exists().await {
                    drop(inner);

                    //hold the lock
                    let mut current_lcu_driver = lcu_driver.inner.write().await;

                    let new_lcu_driver = LcuDriver::connect_wait_lockfile(false).await;

                    current_lcu_driver.replace_inner(new_lcu_driver).await;
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    async fn format_url(&self, url: &str) -> String {
        let inner = self.inner.read().await;

        format!("{}{}", inner.api_base_url, url)
    }

    pub async fn get_current_summoner(&self) -> Result<Summoner> {
        Ok(self.get_endpoint(SummonerEndpoint::Current.info()).await?)
    }

    pub async fn get_champ_select_session(&self) -> Result<ChampSelectSession> {
        Ok(self
            .get_endpoint(ChampSelectEndpoint::Session.info())
            .await?)
    }

    pub async fn get_perks_pages(&self) -> Result<PerksPage> {
        Ok(self
            .get_endpoint(PerksEndpoint::Pages(Method::GET, None).info())
            .await?)
    }

    pub async fn set_perks_page(&self, perks_page: &PerksPage) -> Result<()> {
        let perks_page = serde_json::to_string(perks_page)?;

        Ok(self
            .get_endpoint(PerksEndpoint::Pages(Method::POST, Some(perks_page)).info())
            .await?)
    }

    pub async fn get_endpoint<T: DeserializeOwned>(
        &self,
        endpoint_info: EndpointInfo,
    ) -> Result<T> {
        let inner = self.inner.read().await;

        let mut req = Request::new(
            endpoint_info.method,
            self.format_url(&endpoint_info.url)
                .await
                .parse()
                .expect("Invalid URL"),
        );

        if let Some(headers) = endpoint_info.headers {
            for (k, v) in headers {
                if let Some(k) = k {
                    req.headers_mut().insert(k, v);
                }
            }
        }

        if let Some(body) = endpoint_info.body {
            req.headers_mut()
                .insert("Content-Type", HeaderValue::from_static("application/json"));
            *req.body_mut() = Some(Body::from(body))
        }

        let res = inner.client.execute(req).await?.text().await?;

        dbg!(&res);

        Ok(serde_json::from_str::<T>(&res)?)
    }

    pub async fn league_install_dir(&self) -> PathBuf {
        let inner = self.inner.read().await;

        inner.league_install_dir.clone()
    }
}
