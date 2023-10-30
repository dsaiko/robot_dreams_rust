mod command_csv;
#[cfg(test)]
mod command_csv_tests;
mod commands;

use std::error::Error;
use std::io::Read;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::{env, io, thread};

use commands::{Command, COMMANDS};

pub fn main() -> Result<(), Box<dyn Error>> {
    // read arguments
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => process_interactive(),
        _ => process_cmd_param(&args[1..]),
    }
}

pub fn process_cmd_param(cmd_line: &[String]) -> Result<(), Box<dyn Error>> {
    let cmd_name = cmd_line[0].clone();

    // get selected command
    let Ok(cmd) = Command::from_str(&cmd_name) else {
        return usage("unknown command");
    };

    if cmd.should_exit {
        return Ok(());
    }

    // print header
    eprintln!("Performing {} ({}):", cmd.name.join("|"), cmd.description);

    // read whole multi line input
    let (text, interactive_mode) = if cmd_line.len() == 1 {
        // read text from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        (input, false)
    } else {
        // text are params passed on command line
        (cmd_line[1..].join(" "), true)
    };

    // perform transformation
    cmd.invoke(interactive_mode, &text, &mut io::stdout())
}

pub fn process_interactive() -> Result<(), Box<dyn Error>> {
    // print usage:
    usage("")?;

    thread::scope(|scope| {
        let (tx, rx) = channel::<(Command, String)>();

        // command processor
        scope.spawn(move || {
            for (cmd, text) in rx.iter() {
                let mut stdout = io::stdout().lock();
                if let Err(e) = cmd.invoke(true, &text, &mut stdout) {
                    eprintln!("{}", e);
                    eprintln!();
                } else {
                    eprintln!();
                    eprintln!();
                }
            }
        });

        // command reader
        scope.spawn(|| loop {
            let mut line = String::new();

            let res = io::stdin().read_line(&mut line);
            if res.unwrap_or_default() == 0 {
                drop(tx);
                return;
            };

            let words: Vec<&str> = line.split_whitespace().collect();
            let Some((cmd_name, text)) = words.split_first() else {
                continue;
            };

            let Ok(cmd) = Command::from_str(cmd_name) else {
                eprintln!("unknown command");
                eprintln!();
                continue;
            };

            if cmd.should_exit {
                drop(tx);
                return;
            }

            let text = text.join(" ");

            if let Err(e) = tx.send((cmd, text)) {
                eprintln!("{}", e);
            }
        });
    });

    Ok(())
}

// Prints app usage and returns Error.
fn usage(err: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("COMMAND is one of:");
    for command in COMMANDS {
        eprintln!("\t{}: {}", command.name.join("|"), command.description);
    }

    if err.is_empty() {
        Ok(())
    } else {
        Err(err.into())
    }
}
