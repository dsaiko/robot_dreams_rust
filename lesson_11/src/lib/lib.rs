use std::error::Error;

use serde::{Deserialize, Serialize};

/// Message object
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Message {
    /// Text massage
    Text(String),
    /// Image name, extension and content
    Image(String, String, Vec<u8>),
    /// File name and content
    File(String, Vec<u8>),
}

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

impl Message {
    pub fn from_bytes(bytes: &[u8]) -> Result<Message, Box<dyn Error>> {
        // casting to dyn Error so we do not bind to specific implementation
        let message = bincode::deserialize(bytes)?;
        Ok(message)
    }

    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // casting to dyn Error so we do not bind to specific implementation
        let bytes = bincode::serialize(self)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_text() {
        let source = Message::Text("text message".to_owned());
        let encoded = source.encode().unwrap();
        let target = Message::from_bytes(&encoded[..]).unwrap();

        assert_eq!(source, target);
    }

    #[test]
    fn test_message_image() {
        let source = Message::Image(
            "file_name".to_owned(),
            "png".to_owned(),
            vec![1, 2, 3, 4, 5, 6],
        );
        let encoded = source.encode().unwrap();
        let target = Message::from_bytes(&encoded[..]).unwrap();

        assert_eq!(source, target);
    }

    #[test]
    fn test_message_file() {
        let source = Message::File("random.dat".to_owned(), vec![10, 20, 30, 40, 50, 60]);
        let encoded = source.encode().unwrap();
        let target = Message::from_bytes(&encoded[..]).unwrap();

        assert_eq!(source, target);
    }
}
