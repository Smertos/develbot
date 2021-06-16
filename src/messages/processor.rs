use std::sync::Arc;

use tokio::sync::RwLock;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};

use crate::bot::TwitchChatClient;

use super::COMMAND_PREFIX;
use super::core::Command;
use super::commands::hello_command::HelloCommand;
use sqlx::PgPool;
use crate::database::entity::chat_log_message::ChatLogMessage;
use crate::messages::commands::CommandItem;
use crate::messages::commands::current_command::CurrentCommand;

pub struct MessageProcessor {
    chat_client: Arc<RwLock<TwitchChatClient>>,
    commands: Vec<CommandItem>,
    db_pool: Arc<RwLock<PgPool>>,
}

impl MessageProcessor {
    pub fn new(chat_client: Arc<RwLock<TwitchChatClient>>, db_pool: Arc<RwLock<PgPool>>) -> Self {
        let commands = MessageProcessor::get_commands();

        Self {
            chat_client,
            commands,
            db_pool,
        }
    }

    pub fn get_commands() -> Vec<CommandItem> {
        vec![
            HelloCommand::default(),
            CurrentCommand::default(),
        ]
    }

    pub fn find_matching_command(&self, message: &PrivmsgMessage) -> Option<&CommandItem> {
        for command in self.commands.iter() {
            let command_info = command.get_command_info();
            let slug_with_prefix = format!("{}{}", COMMAND_PREFIX, command_info.get_slug());

            if message.message_text.starts_with(slug_with_prefix.as_str()) {
                return Option::Some(command);
            }
        }

        Option::None
    }

    pub async fn process_message(&self, message: &ServerMessage) -> anyhow::Result<()> {
        match message {
            ServerMessage::ClearChat(message) => {
                log::info!("Chat in channel '{}' has been cleared", message.channel_login);
            },
            ServerMessage::ClearMsg(_) => {},
            ServerMessage::GlobalUserState(_) => {},
            ServerMessage::HostTarget(_) => {},
            ServerMessage::Join(message) => {
                let channel = message.channel_login.clone();
                log::info!("Joined channel '{}'", message.channel_login);

                self.send_privmsg(channel, "Hej".to_string());
            },
            ServerMessage::Notice(x) => {
                log::info!("NOTICE: {}", x.message_text);
            },
            ServerMessage::Part(message) => {
                log::info!("Left channel '{}'", message.channel_login);
            },
            ServerMessage::Ping(_) => {},
            ServerMessage::Pong(_) => {},
            ServerMessage::Privmsg(message) => {
                log::info!("<{}>: {}", message.sender.name, message.message_text);

                // TODO: log message to DB
                // message.server_timestamp
                async {
                    let db_pool = self.db_pool.read().await;
                    let chat_log_message = ChatLogMessage::new(
                        message.sender.login.clone(),
                        message.message_text.clone(),
                        message.server_timestamp.clone()
                    );

                    ChatLogMessage::insert(&db_pool, chat_log_message).await
                }.await?;

                let command = self.find_matching_command(&message);
                if command.is_some() {
                    let command = command.unwrap();
                    command.execute(self, &message);

                    return Ok(());
                }
            },
            ServerMessage::Reconnect(_) => {
                log::debug!("Reconnected");
            },
            ServerMessage::RoomState(_) => {},
            ServerMessage::UserNotice(message) => {
                log::info!("USER NOTICE: {}", message.message_text.clone().unwrap_or("none".to_string()));
            },
            ServerMessage::UserState(_) => {},
            ServerMessage::Whisper(message) => {
                log::info!("<{}> whispered: {}", message.sender.name, message.message_text);
            },
            _ => {}
        }

        Ok(())
    }

    pub fn send_privmsg(&self, channel: String, message: String) {
        let client = self.chat_client.clone();

        tokio::spawn(async move {
            let channel = channel.clone();
            let client = client.read().await;
            let message = message.clone();

            let result = client.privmsg(channel.clone(), message.clone()).await;

            if result.is_err() {
                log::error!("Failed to send privmsg to channel '{}'!", channel);
            } else {
                log::info!("ME: {}", message);
            }
        });
    }
}
