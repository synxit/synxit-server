mod logger;
mod storage;
mod synxit;
mod utils;
mod web;

use std::path::Path;

use log::{debug, info, warn};
use logger::display_copyright;
use synxit::{config::load_config, user::User};
use web::start_server;

#[actix_web::main]
async fn main() {
    display_copyright();

    let args = std::env::args().collect::<Vec<String>>();
    let config = if args.len() >= 2 {
        load_config(Some(Path::new(&args[1])))
    } else {
        load_config(None)
    };

    info!("Starting synxit server...");
    info!("Loading users...");
    for mut user in User::all() {
        user.delete_all_auth_sessions();
        if config
            .tiers
            .iter()
            .find(|tier| tier.id == user.tier)
            .is_none()
        {
            warn!("User {} has an invalid tier", user.username);
        }
        if false {
            user.delete_all_sessions();
        }
    }

    debug!("{:#?}", &config);

    info!("Users loaded");
    info!(
        "Endpoint: http://{}{}/",
        config.network.host,
        if config.network.port != 443 {
            ":".to_owned() + &config.network.port.to_string()
        } else {
            "".to_string()
        }
    );

    start_server().await;
}
