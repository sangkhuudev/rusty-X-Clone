use axum::extract::FromRef;
use rand::rngs::StdRng;
use uchat_crypto::sign::Keys;
use uchat_query::{AsyncConnectionPool, QueryError};

#[derive(FromRef, Clone)]
pub struct AppState {
    pub db_pool: AsyncConnectionPool,
    pub signing_keys: Keys,
    pub rng: StdRng
}

impl AppState {
    pub async fn connect(&self) -> Result<AsyncConnectionPool, QueryError> {
        self.db_pool.get().await
    }
}

pub mod cli {
    use rand::{CryptoRng, RngCore};
    use uchat_crypto::sign::{encode_private_key, EncodedPrivateKey};

    pub fn gen_keys<R>(rng: &mut R) -> color_eyre::Result<(EncodedPrivateKey, Keys)> 
    where R: CryptoRng + RngCore
    {
        let (private_key, keys) = Keys::generate(rng)?;
        let private_key = encode_private_key(private_key)?;
        Ok((private_key, keys))
    }
    pub fn load_keys() -> color_eyre::Result<Keys> {
        let private_key = std::env::var("API_PRIVATE_KEY")
            .wrap_err("Failed to load API_PRIVATE_KEY")
            .suggestion("Please set API_PRIVATE_KEY in .env")?;

        Ok(Keys::from_encoded(private_key)?)
    }
}