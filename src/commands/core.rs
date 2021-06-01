use twitch_irc::message::PrivmsgMessage;

use super::CommandFn;
use super::processor::MessageProcessor;

#[allow(dead_code)]
pub struct Command {
    /// Function that performs the assigned logic of the command
    command_fn: CommandFn,
    /// Description of the command
    description: &'static str,
    /// Readable name of the command
    name: &'static str,
    /// Slug that is used for command parsing
    slug: &'static str,
}

impl Command {
    pub fn new(name: &'static str, description: &'static str, slug: &'static str, command_fn: CommandFn) -> Self {
        Self {
            command_fn,
            description,
            name,
            slug,
        }
    }

    pub fn get_slug(&self) -> &str {
        self.slug.clone()
    }

    pub fn execute(&self, message_processor: &MessageProcessor, message: &PrivmsgMessage) {
        (self.command_fn)(message_processor, message);
    }
}
