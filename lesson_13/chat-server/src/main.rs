use std::collections::HashMap;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use chatlib::Message;
use serde::Deserialize;
use thiserror::Error;
use tracing::{error, info};

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "server_config_default_port")]
    pub port: u16,
    #[serde(default = "server_config_default_hostname")]
    pub hostname: String,
}

fn server_config_default_port() -> u16 {
    11111
}

fn server_config_default_hostname() -> String {
    "localhost".to_owned()
}

struct Server {
    config: ServerConfig,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error")]
    ConfigError(),

    #[error("Unable to listen @ `{0}`")]
    TcpListenerError(String),

    #[error("Error: `{0}`")]
    OtherError(String),
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    Server::new()?.run()
}

impl Server {
    fn new() -> Result<Self> {
        let config = envy::from_env::<ServerConfig>().context(AppError::ConfigError())?;
        Ok(Server { config })
    }

    /// handle client messages and forward them to tx_distributor
    fn handle_client(&self, tx_distributor: Sender<Message>, mut stream: TcpStream) -> Result<()> {
        loop {
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes)?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer)?;

            let msg = Message::from_bytes(&buffer)?;

            match &msg {
                Message::Text(text) => info!(text, "Message"),
                Message::Image(image, ext, _) => info!(image, ext, "Message"),
                Message::File(file, _) => info!(file, "Message"),
            };

            tx_distributor.send(msg)?;
        }
    }

    fn run(&self) -> Result<()> {
        info!("Hello to the Chat Server!");
        info!(self.config.hostname, "HOSTNAME");
        info!(self.config.port, "PORT");

        let server_addr = format!("{}:{}", self.config.hostname, self.config.port);
        let listener = TcpListener::bind(server_addr.clone())
            .context(AppError::TcpListenerError(server_addr))?;

        let clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>> =
            Arc::new(Mutex::new(HashMap::new()));

        thread::scope(|scope| loop {
            let (tx_deregister, rx_deregister) = channel::<SocketAddr>();
            let (tx_distributor, rx_distributor) = channel::<Message>();

            // deregister thread
            let clients_deregister = clients.clone();
            scope.spawn(move || {
                for socket_addr in rx_deregister.iter() {
                    // handler ended
                    // remove this client
                    if let Ok(mut guard) = clients_deregister.lock() {
                        guard.remove(&socket_addr);
                        let count = guard.len();
                        info!(count, "Number of connected clients changed");
                    }
                }
            });

            // distributor thread
            let clients_distributor = clients.clone();
            scope.spawn(move || {
                for msg in rx_distributor.iter() {
                    // handler ended
                    let handler = || -> Result<()> {
                        let mut guard = clients_distributor.lock().map_err(|_| {
                            AppError::OtherError("Unable to lock client list".to_owned())
                        })?;
                        let bytes = msg.encode()?;
                        let len = bytes.len() as u32;

                        for stream in guard.values_mut() {
                            // send message
                            stream.write_all(&len.to_be_bytes())?;
                            stream.write_all(&bytes)?;
                        }

                        Ok(())
                    };

                    if let Err(e) = handler() {
                        error!("{}", e);
                    }
                }
            });

            // listen new connections
            for stream in listener.incoming() {
                let tx_deregister = tx_deregister.clone();
                let tx_distributor = tx_distributor.clone();

                // wrap handler to catch errors
                let handler = || -> Result<()> {
                    let stream = stream?;
                    let client_socket = stream.peer_addr()?;

                    // remember new client
                    let mut guard = clients.lock().map_err(|_| {
                        AppError::OtherError("Unable to lock client list".to_owned())
                    })?;
                    guard.insert(client_socket, stream.try_clone()?);
                    let count = guard.len();
                    info!(count, "Number of connected clients changed");

                    // spawn client handler
                    scope.spawn(move || {
                        _ = self.handle_client(tx_distributor, stream);
                        _ = tx_deregister.send(client_socket);
                    });

                    Ok(())
                };

                let res = handler();
                if let Err(e) = res {
                    error!("{}", e);
                }
            }
        })
    }
}
