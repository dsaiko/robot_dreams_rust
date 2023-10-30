use std::error::Error;
use std::io::Write;
use std::str::FromStr;

use crate::command_csv::command_csv_format;
use slug::slugify;

// Type of command function.
// Function takes input string to manipulate,
// output writer and returns Result.
type CommandFce = fn(bool, &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>>;

// Command definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Command {
    // name of the command
    pub name: &'static [&'static str],

    // description of command
    pub description: &'static str,

    // command function
    command_fce: CommandFce,

    // should diverge or exit
    pub should_exit: bool,
}

impl Command {
    // Invokes the command on input String.
    pub fn invoke(
        &self,
        interactive_mode: bool,
        input: &str,
        out: &mut dyn Write,
    ) -> Result<(), Box<dyn Error>> {
        (self.command_fce)(interactive_mode, input, out)?;
        out.flush()?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseCommandError;

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(cmd_name: &str) -> Result<Self, Self::Err> {
        let cmd_name = cmd_name.trim().to_ascii_lowercase();

        let cmd = COMMANDS
            .iter()
            .find(|c| c.name.contains(&cmd_name.as_str()))
            .ok_or(ParseCommandError)?;

        Ok(cmd.clone())
    }
}

// Definition of supported commands.
pub const COMMANDS: &[Command] = &[
    Command {
        name: &["copy"],
        description: "copies parameter to output without any transformation",
        command_fce: command_clone,
        should_exit: false,
    },
    Command {
        name: &["lowercase"],
        description: "converts text to lowercase",
        command_fce: command_lowercase,
        should_exit: false,
    },
    Command {
        name: &["uppercase"],
        description: "converts text to uppercase",
        command_fce: command_uppercase,
        should_exit: false,
    },
    Command {
        name: &["no-spaces"],
        description: "removes spaces from text",
        command_fce: command_no_spaces,
        should_exit: false,
    },
    Command {
        name: &["slugify"],
        description: "converts the text into a slug",
        command_fce: command_slugify,
        should_exit: false,
    },
    Command {
        name: &["csv"],
        description: "outputs csv as an aligned table",
        command_fce: command_csv_format,
        should_exit: false,
    },
    Command {
        name: &["quit", "exit"],
        description: "quits the app",
        command_fce: command_nop,
        should_exit: true,
    },
];

fn command_clone(_: bool, text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text)?;
    Ok(())
}

fn command_lowercase(_: bool, text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text.to_lowercase())?;
    Ok(())
}

fn command_uppercase(_: bool, text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text.to_uppercase())?;
    Ok(())
}

fn command_no_spaces(_: bool, text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text.replace(' ', ""))?;
    Ok(())
}

fn command_slugify(_: bool, text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", slugify(text))?;
    Ok(())
}

fn command_nop(_: bool, _: &str, _: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    Ok(())
}
