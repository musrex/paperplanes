use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use rand::Rng;

pub fn hash_passwords(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    let parsed_hash = PasswordHash::new(&password_hash);
    Ok(parsed_hash)
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false)
}
