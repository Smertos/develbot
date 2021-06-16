use enum_dispatch::enum_dispatch;
use twitch_irc::message::PrivmsgMessage;

use super::processor::MessageProcessor;

#[allow(dead_code)]
pub struct CommandInfo {
    /// Description of the command
    description: &'static str,
    /// Readable name of the command
    name: &'static str,
    /// Slug that is used for command parsing
    slug: &'static str,
}

impl CommandInfo {
    pub fn new(name: &'static str, description: &'static str, slug: &'static str) -> Self {
        Self {
            description,
            name,
            slug,
        }
    }

    #[allow(dead_code)]
    pub fn get_description(&self) -> &str {
        self.description.clone()
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> &str {
        self.name.clone()
    }

    #[allow(dead_code)]
    pub fn get_slug(&self) -> &str {
        self.slug.clone()
    }
}

#[enum_dispatch(CommandItem)]
pub trait Command {
    fn get_command_info(&self) -> &CommandInfo;

    fn execute(&self, message_processor: &MessageProcessor, message: &PrivmsgMessage);
}
