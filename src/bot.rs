use std::sync::Arc;

use clap::ArgMatches;
use sqlx::PgPool;
use tokio::sync::{mpsc::UnboundedReceiver, RwLock};
use twitch_api2::{helix::channels::ChannelInformation, TwitchClient};
use twitch_irc::{ClientConfig, login::StaticLoginCredentials, message::ServerMessage, TwitchIRCClient, WSSTransport};
use twitch_oauth2::{AppAccessToken, UserToken};

use crate::auth::TokenClient;
use crate::config::{ChannelInfo, Config};
use crate::messages::processor::MessageProcessor;

pub type TwitchChatClient = TwitchIRCClient<WSSTransport, StaticLoginCredentials>;

// There's a lot of Arc+RwLock combos, should think if it's possible to reduce their amount
// Otherwise they'll just keep piling up
pub struct Bot<'a> {
    pub args: Arc<RwLock<ArgMatches<'static>>>,
    pub channel_info: ChannelInfo,
    pub chat_client: Arc<RwLock<TwitchChatClient>>,
    pub chat_incoming_messages: Arc<RwLock<UnboundedReceiver<ServerMessage>>>,
    pub config: Arc<RwLock<Config>>,
    pub db_pool: Arc<RwLock<PgPool>>,
    pub message_processor: Arc<RwLock<MessageProcessor>>,
    pub token_client: Arc<RwLock<TokenClient>>,
    pub twitch_client: TwitchClient<'a, reqwest::Client>,
}

impl<'a> Bot<'a> {
    pub async fn new(
        args: Arc<RwLock<ArgMatches<'static>>>,
        channel_info: ChannelInfo,
        config: Arc<RwLock<Config>>,
        db_pool: Arc<RwLock<PgPool>>,
        token_client: Arc<RwLock<TokenClient>>
    ) -> anyhow::Result<Bot<'a>> {
        // Start token checker client
        TokenClient::start(token_client.clone()).await?;

        // Create Twitch chat IRC client
        let (chat_client, chat_incoming_messages) = Bot::create_irc_client(config.clone(), token_client.clone()).await?;

        let chat_client = Arc::new(RwLock::new(chat_client));
        let chat_incoming_messages = Arc::new(RwLock::new(chat_incoming_messages));

        // Create message processor
        let message_processor = MessageProcessor::new(chat_client.clone(), db_pool.clone());
        let message_processor = Arc::new(RwLock::new(message_processor));

        log::info!("Started bot for channel '{}'", channel_info.channel.as_str());

        Ok(Bot {
            args: args.clone(),
            channel_info,
            chat_client,
            chat_incoming_messages,
            config,
            db_pool,
            message_processor,
            token_client,
            twitch_client: TwitchClient::<reqwest::Client>::default()
        })
    }

    pub async fn create_irc_client(config: Arc<RwLock<Config>>, token_client: Arc<RwLock<TokenClient>>)
        -> anyhow::Result<(TwitchChatClient, UnboundedReceiver<ServerMessage>)>
    {
        log::debug!("Starting bot's chat processor");

        let bot_name = {
            let config = config.read().await;

            config.app_config.twitch.bot_name.clone()
        };

        let token_client_raw = token_client.read().await;
        let bot_token = token_client_raw.user_token.clone().ok_or(anyhow::anyhow!("Can't start chat processor, no user token available"))?;
        let bot_token = bot_token.access_token.secret().clone();

        let client = {
            let credentials = StaticLoginCredentials::new(bot_name, Option::Some(bot_token));
            let chat_client_config = ClientConfig::new_simple(credentials);

            let (incoming_messages, client) = TwitchChatClient::new(chat_client_config);

            Ok((client, incoming_messages))
        };

        if client.is_err() {
            return client.unwrap_err();
        }

        Ok(client.unwrap())
    }

    // Returned data is immutable
    #[allow(dead_code)]
    pub async fn get_app_token(&'a self) -> anyhow::Result<AppAccessToken> {
        let token_client_raw = self.token_client.read().await;
        let token_ref: Option<AppAccessToken> = token_client_raw.app_token.clone();

        let token = token_ref.ok_or(anyhow::Error::msg("this is very bad"))?;

        Ok(token)
    }

    // Returned data is immutable
    #[allow(dead_code)]
    pub async fn get_user_token(&'a self) -> anyhow::Result<UserToken> {
        let token_client_raw = self.token_client.read().await;
        let token_ref: Option<UserToken> = token_client_raw.user_token.clone();

        let token = token_ref.ok_or(anyhow::Error::msg("this is very bad"))?;

        Ok(token)
    }

    #[allow(dead_code)]
    pub async fn get_channel_information(&'a self, channel_name: &'static str) -> anyhow::Result<Option<ChannelInformation>> {
        let client = &self.twitch_client;
        let token = &self.get_app_token().await?;
        let user_id = twitch_api2::types::UserId::from(channel_name);

        let channel_info = client.helix.get_channel_from_id(user_id, token).await?;

        Ok(channel_info)
    }

    pub async fn start_chat_processor(&self) -> anyhow::Result<()> {
        log::debug!("Starting bot's chat processor");

        let chat_incoming_messages = self.chat_incoming_messages.clone();
        let message_processor = self.message_processor.clone();

        let chat_task_handle = tokio::spawn(async move {
            let mut chat_incoming_messages = chat_incoming_messages.write().await;
            let message_processor = message_processor.clone();

            while let Some(message) = chat_incoming_messages.recv().await {
                let message_processor = message_processor.read().await;
                let result = message_processor.process_message(&message).await;

                match result {
                    Err(message) => log::error!("Failed to process message: {}", message),
                    _ => {},
                }
            }
        });

        async {
            let client = self.chat_client.read().await;
            client.join(self.channel_info.channel.clone());
            log::info!("Joined the chat");
        }.await;

        chat_task_handle.await?;

        Ok(())
    }

}
