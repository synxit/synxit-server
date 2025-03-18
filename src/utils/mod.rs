use rand::rngs::OsRng;
use rand::RngCore;

pub fn random_u128() -> u128 {
    let mut rng = OsRng;
    rng.next_u64() as u128 | ((rng.next_u64() as u128) << 64)
}

pub fn u128_to_32_char_hex_string(num: u128) -> String {
    format!("{:032X}", num)
}

pub fn is_32_char_hex_string(s: &str) -> bool {
    s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn current_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
