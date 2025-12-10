use crate::synxit::config::load_config;
use crate::{
    synxit::{config::Config, security::verify_challenge_response},
    utils::{random_u128, u128_to_32_char_hex_string},
};
use std::path::Path;

fn root_dir() -> String {
    let dir = std::env::temp_dir().join("synxit_test_storage");
    dir.to_str().unwrap().to_string()
}

fn setup_config() -> (Config, String) {
    let mut config = Config::default();
    let root_dir = root_dir();
    config.storage.data_dir = root_dir.to_string() + "/data";
    config.storage.log_dir = root_dir.to_string() + "/logs";
    config.storage.temp_dir = root_dir.to_string() + "/temp";
    let config_file_path = root_dir.to_string() + "/test_config.toml";
    std::fs::create_dir_all(&root_dir).unwrap();
    std::fs::write(&config_file_path, toml::to_string(&config).unwrap()).unwrap();
    (
        load_config(Some(Path::new(&config_file_path))),
        config_file_path,
    )
}

fn delete_test_storage() {
    let root_dir = root_dir();
    if std::path::Path::new(&root_dir).exists() {
        std::fs::remove_dir_all(&root_dir).unwrap();
    }
}

#[test]
fn check_login_challenge_works() {
    let challenge = random_u128();
    let password_hash: String = sha256::digest(u128_to_32_char_hex_string(random_u128()));
    let fake_password_hash: String =
        sha256::digest(u128_to_32_char_hex_string(random_u128()) + "fake");
    let response = sha256::digest(format!(
        "{}{}",
        u128_to_32_char_hex_string(challenge),
        &password_hash
    ));
    debug_assert!(verify_challenge_response(
        challenge,
        response.as_str(),
        password_hash.to_string()
    ));

    assert!(!verify_challenge_response(
        challenge,
        response.as_str(),
        fake_password_hash
    ));
}

#[test]
fn load_config_test() {
    let (config, config_file_path) = setup_config();
    assert_eq!(config.storage.data_dir, root_dir() + "/data");
    assert_eq!(config.storage.log_dir, root_dir() + "/logs");
    assert_eq!(config.storage.temp_dir, root_dir() + "/temp");
    std::fs::remove_file(config_file_path).unwrap();
    delete_test_storage();
}
