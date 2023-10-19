use slug::slugify;
use std::env;
use std::error::Error;
use std::process::exit;
struct Command {
    name: &'static str,
    description: &'static str,
    transformation: fn(&String) -> String,
}

const COMMANDS: &[Command] = &[
    Command {
        name: "copy",
        description: "copies parameter to output without any transformation",
        transformation: String::clone,
    },
    Command {
        name: "lowercase",
        description: "converts text to lowercase",
        // note: String::to_lowercase gives me no function or associated item named `to_lowercase`
        // found for struct `std::string::String` in the current scope [E0599]
        // function or associated item not found in `String` Help: the function `to_lowercase`
        // is implemented on `str`
        transformation: transformation_lowercase,
    },
    Command {
        name: "uppercase",
        description: "converts text to uppercase",
        transformation: transformation_uppercase,
    },
    Command {
        name: "no-spaces",
        description: "removes spaces from text",
        transformation: transformation_no_spaces,
    },
    Command {
        name: "slugify",
        description: "converts the text into a slug",
        transformation: transformation_slugify,
    },
];

fn transformation_lowercase(text: &String) -> String {
    text.to_lowercase()
}

fn transformation_uppercase(text: &String) -> String {
    text.to_uppercase()
}

fn transformation_no_spaces(text: &String) -> String {
    text.replace(' ', "")
}

fn transformation_slugify(text: &String) -> String {
    slugify(text)
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let app_name = &args[0];

    if args.len() < 3 {
        error_and_exit(app_name, "missing arguments");
    }

    let command_name = &args[1];

    let Some(command) = COMMANDS
        .iter()
        .find(|c| c.name.to_ascii_lowercase() == command_name.to_ascii_lowercase())
    else {
        error_and_exit(app_name, "unknown command");
    };

    let text = &args[2..].join(" ");
    println!("{}", (command.transformation)(text));

    Ok(())
}

fn error_and_exit(app_name: &String, err: &str) -> ! {
    eprintln!("ERROR: {}", err);
    eprintln!(
        "usage: {} COMMAND TEXT \nwhere COMMAND is one of:",
        app_name
    );
    for command in COMMANDS {
        eprintln!("\t{}: {}", command.name, command.description);
    }

    exit(1);
}
