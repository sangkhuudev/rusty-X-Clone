use std::time::Duration;
use once_cell::sync::OnceCell;
use reqwest::{Client, Response};
use serde::Serialize;
use futures::{
    future::{select, Either},
    pin_mut,
    Future
};
use gloo_timers::future::TimeoutFuture;
// use tokio::time;
// use std::future::Future;

use crate::ROOT_API_URL;

use super::RequestError;

pub static API_CLIENT: OnceCell<ApiClient> = OnceCell::new();

#[derive(Clone, Debug, Default)]
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
        timeout_duration: Duration,
    ) -> Result<Response, RequestError>
    where
        T: Serialize + ?Sized,
    {
        post_json(self.clone(), endpoint, json, timeout_duration).await
    }

    pub fn global() -> &'static ApiClient {
        API_CLIENT.get().expect("api client is not initialized")
    }

    pub fn init() {
        let api_client = reqwest::Client::builder().build().unwrap();
        let api_client = ApiClient::new(api_client);
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
    timeout_duration: Duration,
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
    make_request(api_request, timeout_duration).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn post_json<T>(
    client: ApiClient,
    endpoint: &str,
    json: &T,
    timeout_duration: Duration,
) -> Result<Response, RequestError>
where
    T: Serialize + ?Sized,
{
    let url = make_absolute_url(endpoint);

    let api_request = async { client.inner.post(url).json(json).send().await };
    make_request(api_request, timeout_duration).await
}

fn make_absolute_url(endpoint: &str) -> reqwest::Url {
    let base_url = reqwest::Url::parse(ROOT_API_URL).unwrap();
    base_url.join(endpoint).unwrap()
}

#[test]
fn test_make_absolute_url() {
    let full_url = make_absolute_url("/account/create");
    // Convert the full URL to a string and compare with the expected string
    assert_eq!(full_url.as_str(), "http://127.0.0.1:8000/account/create");
}



async fn make_request(
    api_request: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    timeout: std::time::Duration,
) -> Result<reqwest::Response, RequestError> {
    pin_mut!(api_request);

    let timeout_ms = timeout.as_millis() as u32;
    match select(api_request, TimeoutFuture::new(timeout_ms)).await {
        Either::Left((response, b)) => {
            drop(b);
            response.map_err(RequestError::Request)
        }
        Either::Right((_, request)) => {
            drop(request);
            Err(RequestError::Timeout)
        }
    }
}


#[macro_export]
macro_rules! fetch_json {
    (<$target:ty>, $client:ident, $request:expr) => {{
        use uchat_endpoint::Endpoint;
        use $crate::util::RequestError;
        let duration = std::time::Duration::from_millis(6000);
        let response = $client
            .post_json($request.url(), &$request, duration)
            .await;
        match response {
            Ok(res) => {
                if res.status().is_success() {
                    Ok(res.json::<$target>().await.unwrap())
                } else {
                    let status = res.status();
                    match res.json::<uchat_endpoint::RequestFailed>().await {
                        Ok(payload) => Err(RequestError::BadRequest(payload)),
                        Err(_) => Err(RequestError::BadRequest(uchat_endpoint::RequestFailed {
                            msg: {
                                status
                                    .canonical_reason()
                                    .unwrap_or_else(|| "An error occurred. Please try again.")
                                    .to_string()
                            },
                        })),
                    }
                }
            }
            Err(e) => Err(e),
        }
    }};
}
pub use fetch_json;


