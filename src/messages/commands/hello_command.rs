use twitch_irc::message::PrivmsgMessage;

use crate::messages::commands::CommandItem;
use crate::messages::core::{Command, CommandInfo};
use crate::messages::processor::MessageProcessor;

pub struct HelloCommand {
    command_info: CommandInfo,
}

impl HelloCommand {
    pub fn default() -> CommandItem {
        let command_info = CommandInfo::new(
            "Say Hello",
            "Says 'Hello, <username>' in the chat",
            "hello"
        );

        let command = Self {
            command_info
        };

        CommandItem::HelloCommand(command)
    }
}

impl Command for HelloCommand {
    fn get_command_info(&self) -> &CommandInfo {
        &self.command_info
    }

    fn execute(&self, message_processor: &MessageProcessor, message: &PrivmsgMessage) {
        let channel = message.channel_login.clone();
        message_processor.send_privmsg(channel, format!("Hello, {}!", message.sender.name));
    }
}
