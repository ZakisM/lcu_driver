use std::io::Cursor;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Client, ClientBuilder, Method, Request};
use rustls::ClientConfig;
use serde::de::DeserializeOwned;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::Response;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::endpoints::champ_select::{ChampSelectEndpoint, ChampSelectSession, MySelection};
use crate::endpoints::gameflow::{GameFlowEndpoint, GameFlowSession};
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
    default_req_headers: HeaderMap,
    api_base_url: url::Url,
    websocket_base_url: url::Url,
}

impl LcuDriverInner {
    async fn replace_inner(&mut self, new_inner: LcuDriverInner) {
        self.lcu_process = new_inner.lcu_process;
        self.lockfile = new_inner.lockfile;
        self.client = new_inner.client;
        self.api_base_url = new_inner.api_base_url;
    }
}

pub struct LcuDriver<S> {
    inner: RwLock<LcuDriverInner>,
    _state: S,
    rustls_config: Arc<ClientConfig>,
}

impl LcuDriver<Uninitialized> {
    pub async fn connect() -> Result<LcuDriver<Initialized>> {
        let mut rustls_config = ClientConfig::new();

        let cert_raw = include_bytes!("../certs/riotgames.pem");
        let mut cert = Cursor::new(&cert_raw);

        rustls_config
            .root_store
            .add_pem_file(&mut cert)
            .map_err(|_| LcuDriverError::FailedToReadCertificate)?;

        let lcu_process = LcuProcess::locate().await?;

        let league_install_dir = lcu_process.install_directory();

        let lockfile = Lockfile::load(league_install_dir.join("lockfile")).await?;

        let mut headers = HeaderMap::with_capacity(2);
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Basic {}", lockfile.token))?,
        );

        let client = ClientBuilder::new()
            .default_headers(headers.clone())
            .danger_accept_invalid_certs(true)
            .build()?;

        let api_base_url = url::Url::parse(&format!("https://127.0.0.1:{}", lockfile.port))?;

        let websocket_base_url = url::Url::parse(&format!("wss://localhost:{}/", lockfile.port))?;

        let inner_instance = LcuDriverInner {
            lcu_process,
            lockfile,
            client,
            default_req_headers: headers,
            api_base_url,
            websocket_base_url,
        };

        Ok(LcuDriver {
            inner: RwLock::new(inner_instance),
            rustls_config: Arc::new(rustls_config),
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

    async fn format_url(&self, url: &str) -> Result<url::Url> {
        let inner = self.inner.read().await;

        Ok(inner.api_base_url.join(url)?)
    }

    pub async fn connect_websocket(&self) -> Result<()> {
        let (ws_stream, e) = self.connect_websocket_with_certs().await?;

        Ok(())
    }

    async fn connect_websocket_with_certs(
        &self,
    ) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response<()>)> {
        let inner = self.inner.read().await;

        let connector = tokio_tungstenite::Connector::Rustls(self.rustls_config.clone());

        let uri = http::Uri::from_str(inner.websocket_base_url.as_str())?;

        let mut ws_req = http::Request::builder().uri(&uri).body(())?;

        *ws_req.headers_mut() = inner.default_req_headers.clone();

        let request = ws_req.into_client_request()?;

        let port = request
            .uri()
            .port_u16()
            .or_else(|| match request.uri().scheme_str() {
                Some("wss") => Some(443),
                Some("ws") => Some(80),
                _ => None,
            })
            .expect("Failed to read websocket port");

        let domain = request
            .uri()
            .host()
            .map(|d| d.to_string())
            .expect("Failed to read websocket domain");

        let addr = format!("{}:{}", domain, port);

        let socket = tokio::net::TcpStream::connect(addr).await?;

        let websocket =
            tokio_tungstenite::client_async_tls_with_config(request, socket, None, Some(connector))
                .await?;

        Ok(websocket)
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

    pub async fn get_gameflow_session(&self) -> Result<GameFlowSession> {
        Ok(self
            .get_and_deserialize_endpoint(GameFlowEndpoint::Session.info())
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
            .map_err(|e| LcuDriverError::FailedToSendRequest(e.to_string()))?;

        let status = res.status();

        let res_text = res
            .text()
            .await
            .map_err(|e| LcuDriverError::FailedToReadResponse(e.to_string()))?;

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

        inner.lcu_process.install_directory().to_path_buf()
    }
}
