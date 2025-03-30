mod logger;
mod storage;
mod synxit;
mod utils;
mod web;

use ftail::ansi_escape::TextStyling;
use log::info;
use synxit::{
    config::{load_config, CONFIG},
    user::User,
};
use web::start_server;

#[actix_web::main]
async fn main() {
    let ascii_art = r#"
                                                      88
                                                      ""    ,d
                                                            88
    ,adPPYba,  8b       d8  8b,dPPYba,   8b,     ,d8  88  MM88MMM
    I8[    ""  `8b     d8'  88P'   `"8a   `Y8, ,8P'   88    88
     `"Y8ba,    `8b   d8'   88       88     )888(     88    88
    aa    ]8I    `8b,d8'    88       88   ,d8" "8b,   88    88,
    `"YbbdP"'      Y88'     88       88  8P'     `Y8  88    "Y888
                   d8'
                  d8'       "#;
    println!(
        "{}{} {} {}\n",
        ascii_art.yellow().bold(),
        "(c)".blue().bold(),
        "2021-2025".bright_black(),
        "the synxit developers".yellow().bold()
    );
    load_config();

    let config = CONFIG.get().unwrap();
    info!("Starting synxit server...");
    info!("Loading users...");
    for mut user in User::all() {
        user.delete_all_auth_sessions();
    }
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
