use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
    Argon2,
};
use tracing::instrument;

use password_hash::{PasswordHashString, Salt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Hash error: {0}")]
    HashError(#[from] argon2::password_hash::Error),
    #[error("Invalid password")]
    InvalidPassword,
}
fn new_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}
#[instrument(level = "trace", err, skip_all)]
pub fn hash_password_with_salt<T: AsRef<str>>(
    password: T,
    salt: Salt<'_>
) -> Result<PasswordHashString, Error> {
    let argon2 = Argon2::default();
    let password = password.as_ref().as_bytes();
    let salt = SaltString::from_b64(salt.as_ref())?;

    Ok(argon2.hash_password(password, &salt)?.serialize())
}

#[instrument(level = "trace", err, skip_all)]
pub fn hash_password<T: AsRef<str>>(
    password: T
) -> Result<PasswordHashString, Error> {
    let salt = new_salt();
    hash_password_with_salt(password, salt.as_salt())
}


#[instrument(level = "debug", err, skip_all)]
pub fn verify_password<T: AsRef<str>>(
    password: T,
    hashed_password: &PasswordHash 
) -> Result<(), Error> {
    let argon2 = Argon2::default();
    let password = password.as_ref().as_bytes();

    argon2.verify_password(password, hashed_password)
        .map_err(|_| Error::InvalidPassword)
}

#[tracing::instrument(level = "trace", err, skip_all)]
pub fn deserialize_hash(hash: &str) -> Result<PasswordHash<'_>, Error> {
    Ok(PasswordHash::parse(hash, password_hash::Encoding::B64)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use password_hash::Encoding;
    
    #[test]
    fn hash_password_with_salt_works() {
        let password = "password";
        let salt = new_salt();
        let hashed_password = hash_password_with_salt(password, salt.as_salt()).unwrap();

        assert_ne!(password, hashed_password.as_str());
        assert_eq!(hashed_password.encoding(), Encoding::B64);
    }

    #[test]
    fn matching_passwords_are_properly_verified() {
        let password = "password";
        let hashed_password = hash_password(password).unwrap();
        let hashed_password = PasswordHash::parse(hashed_password.as_str(), Encoding::B64).unwrap();

        verify_password(password, &hashed_password).expect("Failed to match password")
    }
    #[test]
    fn mismatching_passwords_fail_verification() {
        let password = "password";
        let hashed_password = hash_password(password).unwrap();
        let hashed_password = PasswordHash::parse(hashed_password.as_str(), Encoding::B64).unwrap();
        assert!(verify_password("wrong", &hashed_password).is_err());    
    }

    #[test]
    fn deserializes() {
        let password = "password";
        let hashed_password = hash_password(password).unwrap();
        assert!(deserialize_hash(hashed_password.as_str()).is_ok());
    }

}