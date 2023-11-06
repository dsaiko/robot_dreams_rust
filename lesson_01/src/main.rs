use std::error::Error;
use std::io::{self, Write};
use std::{thread, time};

use chrono::Local;

const MESSAGE: &str = "> HELLO RUST <";
const BOX_SIZE: usize = 50;
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S ";

fn build_animation() -> Vec<String> {
    let mut animation = Vec::new();

    let mut message = format!("-{}", MESSAGE);

    // there
    for _ in 0..BOX_SIZE - MESSAGE.len() - 1 {
        let mut frame = message.clone();
        while frame.len() < BOX_SIZE {
            frame += "-";
        }
        animation.push(format!("[{}]", frame));
        message = format!("-{}", message);
    }

    // back
    for i in (0..animation.len()).rev() {
        animation.push(animation[i].clone())
    }

    animation
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let animation = build_animation();

    loop {
        for msg in animation.iter() {
            let now = Local::now();
            print!("{} {}\r", now.format(TIME_FORMAT), msg);

            io::stdout().flush()?;
            thread::sleep(time::Duration::from_millis(100));
        }
    }
}
