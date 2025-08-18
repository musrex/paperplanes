use argon2::{
    Argon2,
    password_hash::{
        Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};

pub fn hash_passwords(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pw_bytes = password.as_bytes();
    let password_hash = argon2.hash_password(pw_bytes, &salt)?.to_string();

    Ok(password_hash)
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash);
    let argon2 = Argon2::default();

    match parsed_hash {
        Ok(ph) => argon2.verify_password(password.as_bytes(), &ph).is_ok(),
        Err(_) => false,
    }
}
