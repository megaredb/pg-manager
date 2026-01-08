use bcrypt::{hash, verify, DEFAULT_COST};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

const ENCRYPTION_KEY: &str = "ZppB2dU5srj32H5erPodjbZohz6TKVFm"; // TODO: Move to config

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap_or_else(|_| password.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub fn encrypt_data(data: &str) -> String {
    let mc = new_magic_crypt!(ENCRYPTION_KEY, 256);
    mc.encrypt_str_to_base64(data)
}

pub fn decrypt_data(encrypted_data: &str) -> Result<String, String> {
    let mc = new_magic_crypt!(ENCRYPTION_KEY, 256);
    mc.decrypt_base64_to_string(encrypted_data)
        .map_err(|_| "Failed to decrypt password".to_string())
}
