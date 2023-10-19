mod command_csv;
#[cfg(test)]
mod command_csv_tests;
mod commands;

use commands::COMMANDS;
use std::env;
use std::error::Error;
use std::io::Read;

pub fn main() -> Result<(), Box<dyn Error>> {
    // read arguments
    let args: Vec<String> = env::args().collect();
    let app_name = &args[0];

    if args.len() != 2 {
        return usage(app_name, "missing arguments");
    }

    // get selected command
    let command_name = &args[1];
    let Some(command) = COMMANDS
        .iter()
        .find(|c| c.name.to_ascii_lowercase() == command_name.to_ascii_lowercase())
    else {
        return usage(app_name, "unknown command");
    };

    // print header
    eprintln!(
        "Performing {} operation ({}):",
        command.name, command.description
    );

    // read input
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    // perform transformation
    command.invoke(&input, &mut std::io::stdout())
}

// Prints app usage and returns Error.
fn usage(app_name: &String, err: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("usage: {} COMMAND\nwhere COMMAND is one of:", app_name);
    for command in COMMANDS {
        eprintln!("\t{}: {}", command.name, command.description);
    }
    Err(err.into())
}
