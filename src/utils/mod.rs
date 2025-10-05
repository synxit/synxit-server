use rand::rngs::OsRng;
use rand::RngCore;

pub fn random_u128() -> u128 {
    let mut rng = OsRng;
    rng.next_u64() as u128 | ((rng.next_u64() as u128) << 64)
}

pub fn u128_to_32_char_hex_string(num: u128) -> String {
    format!("{:032X}", num)
}

pub fn char_hex_string_to_u128(hex: String) -> u128 {
    match u128::from_str_radix(&hex, 16) {
        Ok(num) => num,
        Err(_) => 0,
    }
}

pub fn current_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
