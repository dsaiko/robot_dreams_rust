use std::error::Error;
use std::io::Write;

use crate::command_csv::command_csv_format;
use slug::slugify;

// Type of command function.
// Function takes input string to manipulate,
// output writer and returns Result.
type CommandFce = fn(&str, out: &mut dyn Write) -> Result<(), Box<dyn Error>>;

// Command definition.
pub struct Command {
    // name of the command
    pub name: &'static str,

    // description of command
    pub description: &'static str,

    // command function
    command_fce: CommandFce,
}

impl Command {
    // Invokes the command on input String.
    pub fn invoke(&self, input: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        (self.command_fce)(input, out)
    }
}

// Definition of supported commands.
pub const COMMANDS: &[Command] = &[
    Command {
        name: "copy",
        description: "copies parameter to output without any transformation",
        command_fce: command_clone,
    },
    Command {
        name: "lowercase",
        description: "converts text to lowercase",
        command_fce: command_lowercase,
    },
    Command {
        name: "uppercase",
        description: "converts text to uppercase",
        command_fce: command_uppercase,
    },
    Command {
        name: "no-spaces",
        description: "removes spaces from text",
        command_fce: command_no_spaces,
    },
    Command {
        name: "slugify",
        description: "converts the text into a slug",
        command_fce: command_slugify,
    },
    Command {
        name: "csv",
        description: "outputs csv as an aligned table",
        command_fce: command_csv_format,
    },
];

fn command_clone(text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text)?;
    Ok(())
}

fn command_lowercase(text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text.to_lowercase())?;
    Ok(())
}

fn command_uppercase(text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text.to_uppercase())?;
    Ok(())
}

fn command_no_spaces(text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", text.replace(' ', ""))?;
    Ok(())
}

fn command_slugify(text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(out, "{}", slugify(text))?;
    Ok(())
}
