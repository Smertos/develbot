extern crate anyhow;
extern crate clap;
extern crate enum_dispatch;
extern crate hyper;
extern crate log;
extern crate log4rs;
extern crate oneshot;
extern crate serde;
extern crate sqlx;
extern crate tokio;
extern crate toml;
extern crate twitch_api2;
extern crate twitch_irc;
extern crate twitch_oauth2;

use std::sync::Arc;

use clap::{App, Arg, ArgMatches, crate_authors, crate_description, crate_name, crate_version};
use tokio::sync::RwLock;

use bot::Bot;

use crate::auth::TokenClient;
use crate::config::Config;
use crate::database::connect_db;

mod auth;
mod bot;
mod messages;
mod config;
mod database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::<'static, 'static>::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("app-config")
                .short("c")
                .long("app-config")
                .env("APP_CONFIG_PATH")
                .value_name("APP_CONFIG_PATH")
                .default_value("./configs/config.toml")
                .help("Specifies custom path to bot's primary config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log-config")
                .short("l")
                .long("log-config")
                .env("LOG_CONFIG_PATH")
                .value_name("LOG_CONFIG_PATH")
                .default_value("./configs/logger.toml")
                .help("Specifies custom path to bot's logger config file")
                .takes_value(true),
        );

    // Parse args
    let args: ArgMatches<'static> = app.get_matches();
    let args_arc = Arc::new(RwLock::new(args));

    // Initiate logs
    async {
        let args = args_arc.read().await;
        let log_config = args.value_of("log-config").unwrap();

        log4rs::init_file(log_config, Default::default())?;

        anyhow::Result::<()>::Ok(())
    }.await?;

    log::debug!("{} version {} starting...", crate_name!(), crate_version!());

    // Create & load the config
    let config = Config::from_args(args_arc.clone()).await?;
    let config_arc = Arc::new(RwLock::new(config));

    // Create token checker client
    let token_client = TokenClient::new(config_arc.clone()).await?;
    let token_client_ref: Arc<RwLock<TokenClient>> = Arc::new(RwLock::new(token_client));

    // Create database pool and establish connection
    let db_pool = connect_db(config_arc.clone()).await?;
    let db_pool = Arc::new(RwLock::new(db_pool));

    let channels = async {
        config_arc.read().await.app_config.channels.clone()
    }.await;

    for channel_info in channels {
        let args_arc = args_arc.clone();
        let config_arc = config_arc.clone();
        let db_pool = db_pool.clone();
        let token_client_ref = token_client_ref.clone();

        tokio::spawn(async move {
            let bot = Bot::<'static>::new(
                args_arc,
                channel_info.clone(),
                config_arc,
                db_pool,
                token_client_ref
            ).await;

            if bot.is_err() {
                log::error!("Failed to create bot for channel {}", channel_info.channel.as_str());
                return;
            }

            let result = bot.unwrap().start_chat_processor().await;

            if result.is_err() {
                log::error!("Failed to start/keep alive chat processor for channel {}", channel_info.channel.as_str());
            }
        });
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
