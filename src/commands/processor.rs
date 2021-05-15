use std::sync::Arc;

use tokio::sync::RwLock;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};

use crate::bot::TwitchChatClient;

use super::COMMAND_PREFIX;
use super::core::Command;

pub struct MessageProcessor {
    chat_client: Arc<RwLock<TwitchChatClient>>,
    commands: Vec<Command>,
}

impl MessageProcessor {
    pub fn new(chat_client: Arc<RwLock<TwitchChatClient>>) -> Self {
        let commands = MessageProcessor::get_commands();

        Self {
            chat_client,
            commands
        }
    }

    pub fn get_commands() -> Vec<Command> {
        vec![
            Command::new(
                "Say Hello",
                "Says 'Hello chat' in the chat",
                "hello",
                Arc::new(Box::new(|processor, message| {
                    let channel = message.channel_login.clone();
                    processor.send_privmsg(channel, "Hello, chat!".to_string());
                }))
            ),
        ]
    }

    pub fn find_matching_command(&self, message: &PrivmsgMessage) -> Option<&Command> {
        for command in self.commands.iter() {
            let slug_with_prefix = format!("{}{}", COMMAND_PREFIX, command.get_slug());

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
                let channel = message.channel_login.clone();
                let command = self.find_matching_command(&message);

                if command.is_some() {
                    let command = command.unwrap();
                    command.execute(self, &message);

                    return Ok(());
                }

                if message.message_text.contains("r4ts") && message.message_text.contains("smell") {
                    log::info!("{} said something about r4ts", message.sender.name);
                    let response = "NOOOo. {sender_name}, you smell ever worse D:".replace(
                        "{sender_name}",
                        message.sender.name.as_str()
                    );

                    self.send_privmsg(channel, response.to_string());
                }

                log::info!("<{}>: {}", message.sender.name, message.message_text);
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
        log::debug!("Lets try to send a privmsg");

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
