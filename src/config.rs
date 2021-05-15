use std::fs::File;
use std::io::prelude::*;

use clap::ArgMatches;

use serde::{Deserialize, Serialize};
use twitch_oauth2::{AppAccessToken, UserToken};

#[derive(Debug)]
pub struct Config {
    pub app_config: AppConfig,
    pub config_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub global: GlobalConfig,
    pub twitch: TwitchConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub auth_host: String,
    pub auth_port: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchConfig {
    pub admin: String,
    pub app_access_token: Option<String>,
    pub app_refresh_token: Option<String>,
    pub bot_name: String,
    pub channel: String,
    pub client_id: String,
    pub client_secret: String,
    pub check_every_sec: Option<u64>,
    pub user_access_token: Option<String>,
    pub user_refresh_token: Option<String>,
}

impl Config {
    pub fn from_args(args: &ArgMatches<'static>) -> anyhow::Result<Config> {
        let config_path = args.value_of("app-config").unwrap().to_string(); // Safe unwrap, because we provided the default value at arg def
        let mut config_file = File::open(config_path.clone())?;
        let mut config_contents = String::new();

        config_file.read_to_string(&mut config_contents)?;

        let app_config = toml::from_str(config_contents.as_str())?;

        Ok(Config {
            app_config,
            config_path
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut config_file = File::create(self.config_path.as_str())?;
        let config_contents = toml::to_string_pretty(&self.app_config)?;

        config_file.write_all(config_contents.as_bytes())?;
        config_file.flush()?;

        Ok(())
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        self.save().unwrap_or(());
    }
}

impl Config {
    pub fn set_app_tokens(&mut self, token: &AppAccessToken) -> anyhow::Result<()> {
        self.app_config.twitch.app_access_token = Option::Some(token.access_token.secret().to_string());

        if token.refresh_token.is_some() {
            self.app_config.twitch.app_refresh_token = Option::Some(token.refresh_token.clone().unwrap().secret().to_string());
        }

        self.save()
    }

    pub fn set_user_tokens(&mut self, token: &UserToken) -> anyhow::Result<()> {
        self.app_config.twitch.user_access_token = Option::Some(token.access_token.secret().to_string());

        if token.refresh_token.is_some() {
            self.app_config.twitch.user_refresh_token = Option::Some(token.refresh_token.clone().unwrap().secret().to_string());
        }

        self.save()
    }
}
