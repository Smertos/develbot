use std::sync::Arc;

use twitch_irc::message::PrivmsgMessage;

use self::processor::MessageProcessor;
use std::pin::Pin;
use std::future::Future;

static COMMAND_PREFIX: &'static str = "!";

pub mod core;
pub mod processor;

pub type CommandFn = Arc<Box<dyn Fn(&MessageProcessor, &PrivmsgMessage) -> () + Send + Sync + 'static>>;
