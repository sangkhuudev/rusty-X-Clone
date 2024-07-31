use super::RequestError;
use crate::ROOT_API_URL;
use futures::{
    future::{select, Either},
    pin_mut, Future,
};
use gloo_timers::future::TimeoutFuture;
use once_cell::sync::OnceCell;
use reqwest::{Client, ClientBuilder, Response};
use serde::Serialize;
use std::time::Duration;

pub static API_CLIENT: OnceCell<ApiClient> = OnceCell::new();

#[derive(Debug, Clone, Default)]
pub struct ApiClient {
    pub inner: Client,
}

impl ApiClient {
    pub fn new(client: Client) -> Self {
        Self { inner: client }
    }

    pub async fn post_json<T>(
        &self,
        endpoint: &str,
        json: &T,
        timeout: Duration,
    ) -> Result<Response, RequestError>
    where
        T: Serialize + ?Sized,
    {
        post_json(self.clone(), endpoint, json, timeout).await
    }

    pub fn global() -> &'static ApiClient {
        API_CLIENT.get().expect("api client is not initialized")
    }

    pub fn init() {
        let client = ClientBuilder::new()
            .build()
            .expect("Failed to build client");
        let api_client = ApiClient::new(client);
        if API_CLIENT.set(api_client).is_err() {
            tracing::warn!("Tried to init api client more than once (this is a bug)");
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn post_json<T>(
    client: ApiClient,
    endpoint: &str,
    json: &T,
    timeout: Duration,
) -> Result<Response, RequestError>
where
    T: Serialize + ?Sized,
{
    let url = make_absolute_url(endpoint);
    let api_request = async {
        client
            .inner
            .post(url)
            .fetch_credentials_include()
            .json(json)
            .send()
            .await
    };
    make_request(api_request, timeout).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn post_json<T>(
    client: ApiClient,
    endpoint: &str,
    json: &T,
    timeout: Duration,
) -> Result<Response, RequestError>
where
    T: Serialize + ?Sized,
{
    let url = make_absolute_url(endpoint);

    let api_request = async { client.inner.post(url).json(json).send().await };

    make_request(api_request, timeout).await
}
fn make_absolute_url(endpoint: &str) -> reqwest::Url {
    let base_url = reqwest::Url::parse(ROOT_API_URL).unwrap();
    base_url.join(endpoint).unwrap()
}

#[test]
fn test_make_absolute_url() {
    let full_url = make_absolute_url("/account/create");
    assert_eq!(full_url.as_str(), "http://127.0.0.1:8000/account/create");
}

async fn make_request(
    api_request: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    timeout: Duration,
) -> Result<reqwest::Response, RequestError> {
    pin_mut!(api_request);

    let timeout_ms = timeout.as_millis() as u32;
    match select(api_request, TimeoutFuture::new(timeout_ms)).await {
        Either::Left((response, _)) => response.map_err(RequestError::Request),
        Either::Right((_, _)) => Err(RequestError::Timeout),
    }
}

#[macro_export]
macro_rules! fetch_json {
    (<$target:ty>, $client:ident, $request:expr) => {{
        use dioxus_logger::tracing::{error, info};
        use uchat_endpoint::Endpoint;
        use $crate::util::*;

        let duration = std::time::Duration::from_millis(6000);
        let response = $client.post_json($request.url(), &$request, duration).await;
        match response {
            Ok(res) => {
                info!("Received response status: {}", &res.status());
                if res.status().is_success() {
                    match res.json::<$target>().await {
                        Ok(data) => Ok(data),
                        Err(err) => {
                            error!("Failed to parse JSON response: {:?}", err);
                            Err(RequestError::Request(err))
                        }
                    }
                } else {
                    let err_payload = res.json::<uchat_endpoint::RequestFailed>().await.unwrap();
                    Err(RequestError::BadRequest(err_payload))
                }
            }
            Err(e) => {
                error!("Request failed: {:?}", e);
                Err(e.into()) // Convert reqwest::Error to RequestError
            }
        }
    }};
}

pub use fetch_json;
