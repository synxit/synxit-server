mod erorr;
mod storage;
mod synxit;
mod utils;
mod web;

use synxit::{
    config::{load_config, CONFIG},
    user::User,
};
use web::start_server;

#[actix_web::main]
async fn main() {
    load_config();
    let config = CONFIG.get().unwrap();

    println!("Starting synxit server...");
    println!("Loading users...");
    for mut user in User::all() {
        user.delete_all_auth_sessions();
    }
    println!("Users loaded");
    println!(
        "Endpoint: http://{}:{}/",
        config.network.host,
        if config.network.port != 443 {
            ":".to_owned() + &config.network.port.to_string()
        } else {
            "".to_string()
        }
    );

    start_server().await.expect("Can't start server")
}
