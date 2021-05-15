#![feature(async_closure)]
#![feature(try_trait)]

extern crate anyhow;
extern crate clap;
extern crate hyper;
extern crate log;
extern crate log4rs;
extern crate oneshot;
extern crate sqlx;
extern crate tokio;
extern crate toml;
extern crate twitch_api2;
extern crate twitch_irc;
extern crate twitch_oauth2;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgMatches};

mod auth;
mod bot;
mod commands;
mod config;
// mod database;

use bot::Bot;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;
use crate::auth::TokenClient;

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

    let args: ArgMatches<'static> = app.get_matches();

    log4rs::init_file(args.value_of("log-config").unwrap(), Default::default())?;

    log::debug!("{} version {} starting...", crate_name!(), crate_version!());

    let config = Arc::new(RwLock::new(Config::from_args(&args)?));
    let token_client = TokenClient::new(config.clone()).await?;
    let token_client_ref: Arc<RwLock<TokenClient>> = Arc::new(RwLock::new(token_client));

    let bot: Bot<'static> = Bot::<'static>::new(&args, config, token_client_ref).await?;
    log::debug!("Bot ready for work");

    bot.start_chat_processor().await;

    /* loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    } */

    // let channel_info = bot.get_channel_information("44445592").await?;
    // dbg!(channel_info);

    unreachable!("Application shouldn't finish on it's own")
}
