use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use lesson_09::{Message, ServerConfig};

struct Server {
    config: ServerConfig,
}

fn main() -> Result<(), Box<dyn Error>> {
    Server::new().run()
}

impl Server {
    fn new() -> Self {
        let config = envy::from_env::<ServerConfig>().expect("unable to read app config");
        Server { config }
    }

    /// handle client messages and forward them to tx_distributor
    fn handle_client(
        &self,
        tx_distributor: Sender<Message>,
        mut stream: TcpStream,
    ) -> Result<(), Box<dyn Error>> {
        loop {
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes)?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer)?;

            let msg = Message::from_bytes(&buffer)?;

            let log = match &msg {
                Message::Text(text) => format!("<<< message: {}", text),
                Message::Image(name, ext, _) => format!("<<< image: {}.{}", name, ext),
                Message::File(name, _) => format!("<<< file: {}", name),
            };

            println!("{}", log);

            tx_distributor.send(msg)?;
        }
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        println!("Hello to Lesson09 minimal chat server.");
        println!("\tHOSTNAME = {}", self.config.hostname);
        println!("\tPORT = {}", self.config.port);
        println!();

        let server_addr = format!("{}:{}", self.config.hostname, self.config.port);
        let listener = TcpListener::bind(server_addr.clone())?;

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
                        println!("Connected clients: {}", guard.len());
                    }
                }
            });

            // distributor thread
            let clients_distributor = clients.clone();
            scope.spawn(move || {
                for msg in rx_distributor.iter() {
                    // handler ended
                    let handler = || -> Result<(), Box<dyn Error>> {
                        let mut guard = clients_distributor.lock()?;
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
                        eprintln!("error: {}", e);
                    }
                }
            });

            // listen new connections
            for stream in listener.incoming() {
                let tx_deregister = tx_deregister.clone();
                let tx_distributor = tx_distributor.clone();

                // wrap handler to catch errors
                let handler = || -> Result<(), Box<dyn Error>> {
                    let stream = stream?;
                    let client_socket = stream.peer_addr()?;

                    // remember new client
                    let mut guard = clients.lock()?;
                    guard.insert(client_socket, stream.try_clone()?);
                    println!("Connected clients: {}", guard.len());

                    // spawn client handler
                    scope.spawn(move || {
                        _ = self.handle_client(tx_distributor, stream);
                        _ = tx_deregister.send(client_socket);
                    });

                    Ok(())
                };

                let res = handler();
                if let Err(e) = res {
                    eprintln!("error: {}", e);
                }
            }
        })
    }
}
