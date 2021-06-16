use enum_dispatch::enum_dispatch;
use twitch_irc::message::PrivmsgMessage;

use current_command::CurrentCommand;
use hello_command::HelloCommand;

use crate::messages::core::{Command, CommandInfo};
use crate::messages::MessageProcessor;

pub mod current_command;
pub mod hello_command;

#[enum_dispatch]
pub enum CommandItem {
    CurrentCommand(CurrentCommand),
    HelloCommand(HelloCommand),
}
