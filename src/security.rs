use totp_rs::{Algorithm, Secret, TOTP};

use crate::utils::u128_to_32_char_hex_string;

/// Verify a challenge-response pair using SHA-256 for password login
pub fn verify_challenge_response(challenge: u128, response: &str, password_hash: String) -> bool {
    response
        == sha256::digest(format!(
            "{}{}",
            u128_to_32_char_hex_string(challenge),
            password_hash
        ))
}

/// Verify a TOTP code against a given secret
pub fn verify_totp_code(secret: String, code: &str) -> bool {
    match Secret::Encoded(secret).to_bytes() {
        Ok(bytes) => match TOTP::new(Algorithm::SHA1, 6, 1, 30, bytes) {
            Ok(totp) => totp.check_current(code).unwrap_or(false),
            Err(_) => false,
        },
        Err(_) => false,
    }
}
