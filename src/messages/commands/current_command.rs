use chrono::Datelike;
use chrono::offset::Utc;
use twitch_irc::message::PrivmsgMessage;

use crate::messages::commands::CommandItem;
use crate::messages::core::{Command, CommandInfo};
use crate::messages::processor::MessageProcessor;

fn get_day_suffix(day: u32) -> &'static str {
    match day {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th"
    }
}

pub struct CurrentCommand {
    command_info: CommandInfo,
}

impl CurrentCommand {
    pub fn default() -> CommandItem {
        let command_info = CommandInfo::new(
            "Current Time (UTC)",
            "Returns current datetime in UTC timezone",
            "current"
        );

        let command = Self {
            command_info
        };

        CommandItem::CurrentCommand(command)
    }
}

impl Command for CurrentCommand {
    fn get_command_info(&self) -> &CommandInfo {
        &self.command_info
    }

    fn execute(&self, message_processor: &MessageProcessor, message: &PrivmsgMessage) {
        let channel = message.channel_login.clone();
        let current_time = Utc::now();

        let day_suffix = get_day_suffix(current_time.day());
        let date_format_str = format!("%I:%M:%S %p on %d{} of %B, %G", day_suffix);

        message_processor.send_privmsg(channel, format!("Current datetime: {}", current_time.format(date_format_str.as_str())));
    }
}

#[cfg(test)]
mod tests {
    use super::get_day_suffix;

    #[test]
    fn get_day_suffix_works() {
        assert_eq!(get_day_suffix(1), "st");
        assert_eq!(get_day_suffix(2), "nd");
        assert_eq!(get_day_suffix(3), "rd");
        assert_eq!(get_day_suffix(4), "th");
        assert_eq!(get_day_suffix(5), "th");
        assert_eq!(get_day_suffix(6), "th");
        assert_eq!(get_day_suffix(7), "th");
        assert_eq!(get_day_suffix(8), "th");
        assert_eq!(get_day_suffix(9), "th");
        assert_eq!(get_day_suffix(10), "th");
        assert_eq!(get_day_suffix(11), "th");
        assert_eq!(get_day_suffix(12), "th");
        assert_eq!(get_day_suffix(13), "th");
        assert_eq!(get_day_suffix(14), "th");
        assert_eq!(get_day_suffix(15), "th");
        assert_eq!(get_day_suffix(16), "th");
        assert_eq!(get_day_suffix(17), "th");
        assert_eq!(get_day_suffix(18), "th");
        assert_eq!(get_day_suffix(19), "th");
        assert_eq!(get_day_suffix(20), "th");
        assert_eq!(get_day_suffix(21), "st");
        assert_eq!(get_day_suffix(22), "nd");
        assert_eq!(get_day_suffix(23), "rd");
        assert_eq!(get_day_suffix(24), "th");
        assert_eq!(get_day_suffix(25), "th");
        assert_eq!(get_day_suffix(26), "th");
        assert_eq!(get_day_suffix(27), "th");
        assert_eq!(get_day_suffix(28), "th");
        assert_eq!(get_day_suffix(29), "th");
        assert_eq!(get_day_suffix(30), "th");
        assert_eq!(get_day_suffix(31), "st");
    }
}
