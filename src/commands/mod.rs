use std::sync::Arc;

use twitch_irc::message::PrivmsgMessage;

use self::processor::MessageProcessor;

static COMMAND_PREFIX: &'static str = "~";

pub mod core;
pub mod processor;

pub type CommandFn = Arc<Box<dyn Fn(&MessageProcessor, &PrivmsgMessage) -> () + Send + Sync + 'static>>;
