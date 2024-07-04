use axum::extract::FromRef;
use anyhow::Result;
use rand::rngs::StdRng;
use uchat_crypto::sign::Keys;
use uchat_query::{AsyncConnection, AsyncConnectionPool, QueryError};
pub mod logging;
pub mod router;
#[derive(FromRef, Clone)]
pub struct AppState {
    pub db_pool: AsyncConnectionPool,
    pub signing_keys: Keys,
    pub rng: StdRng
}

impl AppState {
    pub async fn connect(&self) -> Result<AsyncConnection, QueryError> {
        self.db_pool.get().await
    }
}

pub mod cli {
    use anyhow::Context;
    use rand::{CryptoRng, RngCore};
    use uchat_crypto::sign::{encode_private_key, EncodedPrivateKey, Keys};

    pub fn gen_keys<R>(rng: &mut R) -> anyhow::Result<(EncodedPrivateKey, Keys)> 
    where R: CryptoRng + RngCore
    {
        let (private_key, keys) = Keys::generate(rng)?;
        let private_key = encode_private_key(private_key)?;
        Ok((private_key, keys))
    }

    pub fn load_keys() -> anyhow::Result<Keys> {
        let private_key = std::env::var("API_PRIVATE_KEY")
            .with_context(|| format!("set API_PRIVATE_KEY environment variable"))?;
        Ok(Keys::from_encoded(private_key)?)
    }

}