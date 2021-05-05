use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Client, ClientBuilder, Method, Request};
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

use crate::endpoints::champ_select::{ChampSelectEndpoint, ChampSelectSession, MySelection};
use crate::endpoints::perks::{PerksEndpoint, PerksInventory, PerksPage, PerksPages};
use crate::endpoints::summoner::{Summoner, SummonerEndpoint};
use crate::endpoints::EndpointInfo;
use crate::errors::LcuDriverError;
use crate::models::api_error::ApiError;
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
    async fn replace_inner(&mut self, new_inner: LcuDriverInner) {
        self.lcu_process = new_inner.lcu_process;
        self.lockfile = new_inner.lockfile;
        self.client = new_inner.client;
        self.api_base_url = new_inner.api_base_url;
        self.league_install_dir = new_inner.league_install_dir;
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

    async fn connect_wait_no_reconnect() -> LcuDriver<Initialized> {
        loop {
            if let Ok(lcu_driver) = LcuDriver::connect().await {
                //Check that we can actually connect to the client
                if lcu_driver.get_current_summoner().await.is_ok() {
                    return lcu_driver;
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn connect_wait() -> Arc<LcuDriver<Initialized>> {
        loop {
            if let Ok(lcu_driver) = LcuDriver::connect().await {
                //Check that we can actually connect to the client
                if lcu_driver.get_current_summoner().await.is_ok() {
                    let pointer = Arc::new(lcu_driver);
                    LcuDriver::start_lockfile_watching(pointer.clone());

                    return pointer;
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

impl LcuDriver<Initialized> {
    /*
        Spawn a task that will check if the lockfile has changed. If the lockfile exists but has
        new data, then it will be updated. However, if the lockfile no longer exists then any
        calls made from the driver will block until the client process is detected and the lockfile
        exists.
    */
    fn start_lockfile_watching(lcu_driver: Arc<LcuDriver<Initialized>>) {
        tokio::task::spawn(async move {
            loop {
                let current_inner = lcu_driver.inner.read().await;

                if !current_inner.lockfile.exists().await
                    || current_inner.lockfile.contents_changed().await
                {
                    drop(current_inner);

                    //hold the lock preventing any api calls from running
                    let mut current_lcu_driver = lcu_driver.inner.write().await;

                    let new_lcu_driver = LcuDriver::connect_wait_no_reconnect().await;

                    current_lcu_driver
                        .replace_inner(new_lcu_driver.into_inner())
                        .await;
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    fn into_inner(self) -> LcuDriverInner {
        self.inner.into_inner()
    }

    async fn format_url(&self, url: &str) -> String {
        let inner = self.inner.read().await;

        format!("{}{}", inner.api_base_url, url)
    }

    pub async fn get_current_summoner(&self) -> Result<Summoner> {
        Ok(self
            .get_and_deserialize_endpoint(SummonerEndpoint::Current.info())
            .await?)
    }

    pub async fn get_champ_select_session(&self) -> Result<ChampSelectSession> {
        Ok(self
            .get_and_deserialize_endpoint(ChampSelectEndpoint::Session.info())
            .await?)
    }

    pub async fn get_perks_inventory(&self) -> Result<PerksInventory> {
        Ok(self
            .get_and_deserialize_endpoint(PerksEndpoint::Inventory.info())
            .await?)
    }

    pub async fn get_perks_pages(&self) -> Result<PerksPages> {
        let pages = self
            .get_and_deserialize_endpoint(PerksEndpoint::Pages(Method::GET, None).info())
            .await?;

        Ok(PerksPages { pages })
    }

    pub async fn set_perks_page(&self, perks_page: &PerksPage) -> Result<()> {
        let perks_page = serde_json::to_string(perks_page)?;

        self.get_endpoint(PerksEndpoint::Pages(Method::POST, Some(perks_page)).info())
            .await?;

        Ok(())
    }

    pub async fn delete_perks_page(&self, page_id: isize) -> Result<()> {
        self.get_endpoint(PerksEndpoint::PagesId(Method::DELETE, page_id).info())
            .await?;

        Ok(())
    }

    pub async fn set_session_my_selection(&self, my_selection: &MySelection) -> Result<()> {
        let my_selection = serde_json::to_string(my_selection)?;

        self.get_endpoint(ChampSelectEndpoint::SessionMySelection(&my_selection).info())
            .await?;

        Ok(())
    }

    pub async fn get_endpoint(&self, endpoint_info: EndpointInfo) -> Result<String> {
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

        let res = inner
            .client
            .execute(req)
            .await
            .map_err(|_| LcuDriverError::FailedToSendRequest)?;

        let status = res.status();

        let res_text = res
            .text()
            .await
            .map_err(|_| LcuDriverError::FailedToReadResponse)?;

        dbg!(&res_text);

        if !status.is_success() {
            let err = serde_json::from_str::<ApiError>(&res_text)?;

            Err(err.into())
        } else {
            Ok(res_text)
        }
    }

    pub async fn get_and_deserialize_endpoint<T: DeserializeOwned>(
        &self,
        endpoint_info: EndpointInfo,
    ) -> Result<T> {
        let res = self.get_endpoint(endpoint_info).await?;

        Ok(serde_json::from_str::<T>(&res)?)
    }

    pub async fn league_install_dir(&self) -> PathBuf {
        let inner = self.inner.read().await;

        inner.league_install_dir.clone()
    }
}
