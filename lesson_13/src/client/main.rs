use std::{fs, io, thread};
use std::error::Error;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::iter::repeat_with;
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use image::ImageFormat;
use lesson_09::{Message, ServerConfig};
use serde::Deserialize;

struct Client {
    config: ClientConfig,
    stream: Mutex<Option<TcpStream>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    Client::new().run()
}

/// Client env configuration.
#[derive(Deserialize, Debug)]
pub struct ClientConfig {
    #[serde(flatten)]
    pub server: ServerConfig,
    #[serde(default = "client_config_default_username")]
    pub username: String,
}

/// Randomly generated username.
fn client_config_default_username() -> String {
    format!(
        "user-{}",
        repeat_with(fastrand::alphanumeric)
            .take(5)
            .collect::<String>()
    )
}

impl Client {
    /// initialize new instance
    fn new() -> Self {
        let config = envy::from_env::<ClientConfig>().expect("unable to read app config");
        Client {
            config,
            stream: Mutex::new(None), // no stream at init
        }
    }

    /// get existing stream connection
    fn get_stream(&self) -> Option<TcpStream> {
        if let Ok(mut guard) = self.stream.lock() {
            if let Some(stream) = &mut *guard {
                if let Ok(stream) = stream.try_clone() {
                    return Some(stream);
                }
            }
        }
        None
    }

    /// create a new server stream
    fn create_stream(&self) -> Result<TcpStream, Box<dyn Error + '_>> {
        let mut guard = self.stream.lock()?;
        let stream = TcpStream::connect(format!(
            "{}:{}",
            self.config.server.hostname, self.config.server.port
        ))?;
        *guard = Some(stream.try_clone()?);
        Ok(stream)
    }

    /// send message to the stream
    fn send_message(&self, stream: &mut TcpStream, msg: Message) -> Result<(), Box<dyn Error>> {
        // send message
        let bytes = msg.encode()?;
        let len = bytes.len() as u32;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&bytes)?;

        Ok(())
    }

    /// will read replies from server
    /// this fce will end on a read error
    fn read_server_replies(&self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        loop {
            // read message
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes)?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer)?;

            let msg = Message::from_bytes(&buffer)?;

            // process message
            match msg {
                Message::Text(text) => println!("<<< {}", text),
                Message::Image(name, ext, bytes) => {
                    print!("<<< image: {}.{}", name, ext);
                    match self.process_incoming_image(name, ext, bytes) {
                        Ok(file) => println!(" -> {}", file),
                        Err(e) => println!(" error: {}", e),
                    }
                }
                Message::File(name, bytes) => {
                    print!("<<< file: {}", name);
                    match self.process_incoming_file(name, bytes) {
                        Ok(file) => println!(" -> {}", file),
                        Err(e) => println!(" error: {}", e),
                    }
                }
            };
        }
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        println!("Hello to Lesson09 minimal chat client.");
        println!("\tUSERNAME = {}", self.config.username);
        println!("\tHOSTNAME = {}", self.config.server.hostname);
        println!("\tPORT = {}", self.config.server.port);
        println!();

        thread::scope(|scope| {
            let (tx, rx) = channel::<Message>();

            // command processor
            scope.spawn(move || {
                for cmd in rx.iter() {
                    let log = match &cmd {
                        Message::Text(msg) => msg.clone(),
                        Message::Image(name, ext, _) => format!("Sending image: {}.{}", name, ext),
                        Message::File(name, _) => format!("Sending file: {}", name),
                    };
                    println!(">>> {}", log);

                    let mut stream = if let Some(stream) = self.get_stream() {
                        stream
                    } else {
                        let stream = self.create_stream();
                        let Ok(stream) = stream else {
                            eprintln!("error: {}", stream.err().unwrap());
                            continue;
                        };

                        let reply_stream = stream.try_clone();
                        let Ok(reply_stream) = reply_stream else {
                            eprintln!("error: {}", reply_stream.err().unwrap());
                            continue;
                        };
                        scope.spawn(move || {
                            if let Err(response) = self.read_server_replies(reply_stream) {
                                eprintln!("disconnecting reply reader: {}", response)
                            }
                        });

                        stream
                    };

                    if let Err(e) = self.send_message(&mut stream, cmd) {
                        eprintln!("error: {}", e);

                        // reset stream
                        if let Ok(mut guard) = self.stream.lock() {
                            *guard = None;
                        }
                    }
                }
            });

            // command reader
            let tx_command = tx.clone();
            scope.spawn(|| {
                loop {
                    let mut cmd_line = String::new();

                    let res = io::stdin().read_line(&mut cmd_line);
                    if res.unwrap_or_default() == 0 {
                        break;
                    };

                    let cmd_line = cmd_line.trim();
                    if cmd_line.is_empty() {
                        continue;
                    }

                    if cmd_line == ".quit" {
                        break;
                    }

                    let cmd = self.message_from_command_line(cmd_line);
                    let Ok(cmd) = cmd else {
                        eprintln!("error: {}", cmd.err().unwrap());
                        continue;
                    };

                    // file or image command can produce multiple messages
                    // command: .file a.dat b.dat
                    for msg in cmd {
                        if let Err(e) = tx_command.send(msg) {
                            eprintln!("error: {}", e);
                        }
                    }
                }

                drop(tx_command);
                exit(0);
            });

            // send initial greeting
            _ = tx.send(Message::Text(format!(
                "Hello from {}",
                self.config.username
            )));
        });

        Ok(())
    }

    /// constructs message from command line
    pub fn message_from_command_line(
        &self,
        cmd_line: &str,
    ) -> Result<Vec<Message>, Box<dyn Error>> {
        let words = cmd_line.split_whitespace().collect::<Vec<_>>();

        let Some((cmd, params)) = words.split_first() else {
            return Err("no command supplied".into());
        };

        let message = match *cmd {
            ".file" => self.create_file_message(params)?,
            ".image" => self.create_image_message(params)?,
            _ => vec![Message::Text(cmd_line.to_owned())],
        };

        Ok(message)
    }

    /// construct file message f
    fn create_file_message(&self, files: &[&str]) -> Result<Vec<Message>, Box<dyn Error>> {
        let mut messages = vec![];
        for file in files {
            let bytes = fs::read(file)?;
            messages.push(Message::File(file.to_string(), bytes))
        }

        Ok(messages)
    }

    /// construct image message
    /// images will be converted to png when received
    fn create_image_message(&self, files: &[&str]) -> Result<Vec<Message>, Box<dyn Error>> {
        let mut messages = vec![];
        for file in files {
            // read image as bytes
            let bytes = fs::read(file)?;
            let Some(name) = Path::new(file).file_stem().and_then(OsStr::to_str) else {
                return Err("unable to read file name".into());
            };

            let Some(ext) = Path::new(file).extension().and_then(OsStr::to_str) else {
                return Err("unable to read file extension".into());
            };

            messages.push(Message::Image(name.to_owned(), ext.to_owned(), bytes))
        }

        Ok(messages)
    }

    /// convert and save incoming image
    fn process_incoming_image(
        &self,
        name: String,
        ext: String,
        bytes: Vec<u8>,
    ) -> Result<String, Box<dyn Error>> {
        let path = Path::new("incoming_images");
        fs::create_dir_all(path)?;

        let Some(format) = ImageFormat::from_extension(ext.clone()) else {
            return Err(format!("unknown image format: .{}", ext).into());
        };

        let img = image::load_from_memory_with_format(&bytes, format)?;

        let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let output_file = path.join(format!("{}-{}.png", t, name));
        let output_file = match output_file.to_str() {
            None => return Err("unable to construct output file name".into()),
            Some(s) => s.to_owned(),
        };
        img.save(output_file.clone())?;

        Ok(output_file)
    }

    /// convert and save incoming file
    fn process_incoming_file(
        &self,
        name: String,
        bytes: Vec<u8>,
    ) -> Result<String, Box<dyn Error>> {
        let path = Path::new("incoming_files");
        fs::create_dir_all(path)?;

        let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let output_file = path.join(format!("{}-{}", t, name));
        let output_file = match output_file.to_str() {
            None => return Err("unable to construct output file name".into()),
            Some(s) => s.to_owned(),
        };

        fs::write(output_file.clone(), bytes)?;

        Ok(output_file)
    }
}

#[cfg(test)]
mod tests {
    use lesson_09::Message;

    use crate::Client;

    #[test]
    fn test_command_text() {
        let source = Client::new()
            .message_from_command_line("some text")
            .unwrap();
        assert_eq!(source, vec![Message::Text("some text".to_owned())]);
    }

    #[test]
    fn test_command_text_one_word() {
        let source = Client::new().message_from_command_line("some").unwrap();
        assert_eq!(source, vec![Message::Text("some".to_owned())]);
    }
}
