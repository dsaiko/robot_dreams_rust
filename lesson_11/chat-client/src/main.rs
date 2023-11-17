use std::error::Error;
use std::io::{Read, Write};
use std::iter::repeat_with;
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io, thread};

use image::ImageFormat;
use serde::Deserialize;
use tracing::{error, info};

use chatlib::Message;

struct Client {
    config: ClientConfig,
    stream: Mutex<Option<TcpStream>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    Client::new().run()
}

/// Client env configuration.
#[derive(Deserialize, Debug)]
pub struct ClientConfig {
    #[serde(default = "server_config_default_port")]
    pub port: u16,
    #[serde(default = "server_config_default_hostname")]
    pub hostname: String,
    #[serde(default = "client_config_default_username")]
    pub username: String,
}

fn server_config_default_port() -> u16 {
    11111
}

fn server_config_default_hostname() -> String {
    "localhost".to_owned()
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
        let stream = TcpStream::connect(format!("{}:{}", self.config.hostname, self.config.port))?;
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
                Message::Text(text) => info!(text, "Incoming"),
                Message::Image(image, ext, bytes) => {
                    info!(image, ext, "Incoming");
                    match self.process_incoming_image(image, ext, bytes) {
                        Ok(name) => info!(name, "Image saved"),
                        Err(e) => error!("Unable to save image: {}", e),
                    }
                }
                Message::File(file, bytes) => {
                    info!(file, "incoming");
                    match self.process_incoming_file(file, bytes) {
                        Ok(name) => info!(name, "File saved"),
                        Err(e) => error!("Unable to save file: {}", e),
                    }
                }
            };
        }
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Hello to the Chat Client!");
        info!(self.config.username, "USERNAME");
        info!(self.config.hostname, "HOSTNAME");
        info!(self.config.port, "PORT");

        thread::scope(|scope| {
            let (tx, rx) = channel::<Message>();

            // command processor
            scope.spawn(move || {
                for cmd in rx.iter() {
                    match &cmd {
                        Message::Text(text) => info!(text, "Outgoing"),
                        Message::Image(image, ext, _) => info!(image, ext, "Outgoing"),
                        Message::File(file, _) => info!(file, "Outgoing"),
                    };

                    let mut stream = if let Some(stream) = self.get_stream() {
                        stream
                    } else {
                        let stream = self.create_stream();
                        let Ok(stream) = stream else {
                            error!("{}", stream.err().unwrap());
                            continue;
                        };

                        let reply_stream = stream.try_clone();
                        let Ok(reply_stream) = reply_stream else {
                            error!("{}", reply_stream.err().unwrap());
                            continue;
                        };
                        scope.spawn(move || {
                            if let Err(response) = self.read_server_replies(reply_stream) {
                                error!("Server disconnected: {}", response);
                            }
                        });

                        stream
                    };

                    if let Err(e) = self.send_message(&mut stream, cmd) {
                        error!("{}", e);

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

                    match cmd_line {
                        ".quit" => break,
                        ".ls" => {
                            self.ls();
                            continue;
                        }
                        &_ => {}
                    }

                    let cmd = self.message_from_command_line(cmd_line);
                    let Ok(cmd) = cmd else {
                        error!("{}", cmd.err().unwrap());
                        continue;
                    };

                    // file or image command can produce multiple messages
                    // command: .file a.dat b.dat
                    for msg in cmd {
                        if let Err(e) = tx_command.send(msg) {
                            error!("{}", e);
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

        let message: Vec<Message> = match *cmd {
            ".file" => params
                .iter()
                .map(|filename| Message::new_file_message(filename))
                .collect::<Result<Vec<_>, _>>()?,
            ".image" => params
                .iter()
                .map(|filename| Message::new_image_message(filename))
                .collect::<Result<Vec<_>, _>>()?,
            _ => vec![Message::new_text_message(cmd_line)],
        };

        Ok(message)
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
    fn ls(&self) {
        let paths = fs::read_dir("./");

        let Ok(paths) = paths else {
            error!("{}", paths.err().unwrap());
            return;
        };

        for path in paths.flatten() {
            println!("\t{}", path.path().display());
        }
    }
}
