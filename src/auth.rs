use std::{thread, time::Duration};
use std::sync::Arc;

use tokio::sync::RwLock;
use twitch_oauth2::{AccessToken, AppAccessToken, client::reqwest_http_client, ClientId, ClientSecret, RedirectUrl, RefreshToken, Scope, TwitchToken, UserToken};
use twitch_oauth2::tokens::UserTokenBuilder;

use crate::config::Config;

pub struct TokenClient {
    pub app_token: Option<AppAccessToken>,
    pub config: Arc<RwLock<Config>>,
    pub check_interval: u64,
    is_running: bool,
    pub user_token: Option<UserToken>,
    pub validity_period: u64,
}

unsafe impl Send for TokenClient {}

impl TokenClient {
    pub async fn new(config: Arc<RwLock<Config>>) -> anyhow::Result<Self> {
        let config_clone = config.clone();
        let lock = config_clone.read().await;
        let check_interval = lock.app_config.twitch.check_every_sec.unwrap_or(15);

        Ok(TokenClient {
            app_token: Option::None,
            check_interval,
            config,
            is_running: false,
            user_token: Option::None,
            validity_period: check_interval * 2
        })
    }

    pub async fn check_app_token(token_client: &mut TokenClient, config: &mut Config, token: Option<AppAccessToken>, validity_period: u64) -> anyhow::Result<()> {
        if token.as_ref().is_some() && !token.as_ref().unwrap().access_token.secret().is_empty() {
            match token.unwrap().validate_token(reqwest_http_client).await {
                Err(_) => {
                    // log::debug!("Failed to validate token");
                    TokenClient::get_app_token(token_client, config).await?;
                },
                Ok(validated_token) => {
                    if validated_token.expires_in.as_secs() <= validity_period {
                        // log::debug!("dying token, getting a new one");
                        TokenClient::get_app_token(token_client, config).await?;
                    } else {
                        // log::debug!("Token good");
                    }
                },
            }
        } else {
            log::debug!("No token provided, getting a new one...");
            TokenClient::get_app_token(token_client, config).await?;
        }

        log::debug!("Checked twitch token");

        Ok(())
    }

    pub async fn get_app_token(token_client: &mut TokenClient, config: &mut Config) -> anyhow::Result<()> {
        let client_id = ClientId::new(config.app_config.twitch.client_id.clone());
        let client_secret = ClientSecret::new(config.app_config.twitch.client_secret.clone());
        let scopes = Scope::all();

        let token = AppAccessToken::get_app_access_token(reqwest_http_client, client_id, client_secret, scopes).await?;

        config.set_app_tokens(&token)?;
        token_client.app_token = Option::Some(token);

        Ok(())
    }

    pub async fn get_user_token(token_client: &mut TokenClient, config: &mut Config) -> anyhow::Result<()> {
        let client_id = ClientId::new(config.app_config.twitch.client_id.clone());
        let client_secret = ClientSecret::new(config.app_config.twitch.client_secret.clone());

        let user_access_token = config.app_config.twitch.user_access_token.clone();
        let user_refresh_token = config.app_config.twitch.user_refresh_token.clone();

        if user_access_token.is_some() && user_refresh_token.is_some() {
            let user_access_token = user_access_token.unwrap();
            let user_refresh_token = user_refresh_token.unwrap();

            if user_access_token.len() > 0 && user_refresh_token.len() > 0 {
                let user_access_token = AccessToken::new(user_access_token);
                let user_refresh_token = RefreshToken::new(user_refresh_token);

                match UserToken::from_existing(reqwest_http_client, user_access_token, user_refresh_token, client_secret.clone()).await {
                    Ok(token) => {
                        token_client.user_token = Option::Some(token);
                        return Ok(());
                    },
                    Err(_) => {},
                };
            }
        }

        let bot_name = config.app_config.twitch.bot_name.clone();
        let auth_host = config.app_config.global.auth_host.clone();
        let auth_port = config.app_config.global.auth_port.clone();
        let host_port = format!("{}:{}", auth_host, auth_port);
        let redirect_url_src = String::from(format!("http://{}/oauth-receive/", host_port.clone()));
        let redirect_url = RedirectUrl::new(redirect_url_src.clone())?;

        log::debug!("redirect url: {}", redirect_url_src);

        let mut builder = UserTokenBuilder::new(client_id.clone(), client_secret.clone(), redirect_url.clone())?
            .set_scopes(Scope::all());
        let (authorize_url, csrf) = builder.generate_url();

        log::info!("PLEASE LOGIN as {} at given URL:\n{}", bot_name, authorize_url.to_string());

        let (sender, receiver) = oneshot::channel::<UserToken>();

        let server = match tiny_http::Server::http(host_port.clone()) {
            Ok(server) => server,
            Err(_) => return Err(anyhow::anyhow!("Failed to setup oauth callback server")),
        };

        for request in server.incoming_requests() {
            fn easy_respond(req: tiny_http::Request, data: &str, status: Option<u16>) -> anyhow::Result<()> {
                let response = tiny_http::Response::from_string(data)
                    .with_status_code(status.unwrap_or(200));

                req.respond(response)?;

                Ok(())
            }

            log::debug!("url: {}", request.url());

            let full_url = format!("http://{}{}", host_port, request.url());
            let url_dangerous = url::Url::parse(full_url.as_str());

            if url_dangerous.is_err() {
                easy_respond(request, "Couldn't parse uri", Option::Some(500))?;
                continue;
            }

            let url = url_dangerous.unwrap();
            let query_params = url.query_pairs();

            let mut code: Option<String> = Option::None;
            let mut state: Option<String> = Option::None;

            for (key, value) in query_params {
                if key == "code" {
                    code = Option::Some(value.clone().into());
                }

                if key == "state" {
                    state = Option::Some(value.clone().into());
                }
            }

            if code.as_ref().is_none() {
                easy_respond(request, "Couldn't find query parameter 'code'", Option::Some(400))?;
                continue;
            }

            if state.as_ref().is_none() {
                easy_respond(request, "Couldn't find query parameter 'state'", Option::Some(400))?;
                continue;
            }

            let mut builder = UserTokenBuilder::new(client_id.clone(), client_secret.clone(), redirect_url.clone())?;
            builder.set_csrf(csrf.clone());

            let token = builder.get_user_token(reqwest_http_client, state.unwrap().as_str(), code.unwrap().as_str()).await;

            if token.is_err() {
                log::error!("Token error:  {}", token.unwrap_err().to_string());
                easy_respond(request, "Failed to acquire token", Option::Some(500))?;
                continue;
            }

            let token = token.unwrap();
            let send_result = sender.send(token);

            if send_result.is_err() {
                panic!("Failed to send resulting user token back");
            }

            easy_respond(request, "Got the token, good to go", Option::None)?;
            break;
        }

        let check_period = Duration::from_secs(1);
        let token: UserToken = loop {
            match receiver.recv_timeout(check_period) {
                Ok(token) => {
                    log::debug!("Received code & state, shutting down oauth endpoint");

                    break token;
                },
                _ => {},
            }
        };

        config.set_user_tokens(&token)?;
        token_client.user_token = Option::Some(token);

        Ok(())
    }

    pub async fn start(this: Arc<RwLock<TokenClient>>) -> anyhow::Result<()> {
        let period = {
            let this_clone = this.clone();
            let mut this_lock = this_clone.write().await;

            let app_token = {
                let config_clone = this_lock.config.clone();
                let mut config_lock = config_clone.write().await;

                let client_id = ClientId::new(config_lock.app_config.twitch.client_id.clone());
                let client_secret = ClientSecret::new(config_lock.app_config.twitch.client_secret.clone());
                let scopes = Scope::all();

                let app_access_token = AccessToken::new(config_lock.app_config.twitch.app_access_token.clone().unwrap_or(String::from("")));
                let app_refresh_token = RefreshToken::new(config_lock.app_config.twitch.app_refresh_token.clone().unwrap_or(String::from("")));

                let app_token = AppAccessToken::from_existing_unchecked(app_access_token, app_refresh_token, client_id.clone(), client_secret.clone(), Option::None, Option::Some(scopes), Option::None);

                TokenClient::get_user_token(&mut this_lock, &mut config_lock).await?;

                app_token
            };

            this_lock.app_token = Option::Some(app_token);
            this_lock.is_running = true;

            let config_clone = this_lock.config.clone();
            let mut config_lock = config_clone.write().await;

            let token = this_lock.app_token.clone();
            let validity_period = this_lock.validity_period.clone();

            let _ = TokenClient::check_app_token(&mut this_lock, &mut config_lock, token, validity_period).await?;

            log::debug!("First token check completed!");

            Duration::from_secs(this_lock.check_interval)
        };

        thread::spawn({
            let local_this = this.clone();

            move || async move {
                let is_running = || async {
                    let this_clone = local_this.clone();
                    let lock = this_clone.read().await;

                    lock.is_running
                };

                while is_running().await {
                    let this_clone = local_this.clone();
                    let mut this_lock = this_clone.write().await; // lifetime 'a != lifetime 'static

                    let config_clone = this_lock.config.clone();
                    let mut config_lock = config_clone.write().await;

                    let token = this_lock.app_token.clone();
                    let validity_period = this_lock.validity_period.clone();

                    let result = TokenClient::check_app_token(&mut this_lock, &mut config_lock, token, validity_period).await;

                    if result.is_err() {
                        return ()
                    }

                    thread::sleep(period);
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) -> () {
        self.is_running = false;
    }
}

impl<'a> Drop for TokenClient {
    fn drop(&mut self) {
        self.stop();
    }
}
