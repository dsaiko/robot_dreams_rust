use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use image::ImageFormat;
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

impl Message {
    /// construct a new message from a file
    pub fn new_file_message(file: &str) -> Result<Message, Box<dyn Error>> {
        let bytes = fs::read(file)?;
        Ok(Message::File(file.to_string(), bytes))
    }

    /// construct a new text message
    pub fn new_text_message(text: &str) -> Message {
        Message::Text(text.to_owned())
    }

    /// construct a new message from image file
    pub fn new_image_message(file: &str) -> Result<Message, Box<dyn Error>> {
        // read image as bytes
        let bytes = fs::read(file)?;
        let Some(name) = Path::new(file).file_stem().and_then(OsStr::to_str) else {
            return Err("unable to read file name".into());
        };

        let Some(ext) = Path::new(file).extension().and_then(OsStr::to_str) else {
            return Err("unable to read file extension".into());
        };

        // validate image format
        let Some(format) = ImageFormat::from_extension(ext) else {
            return Err(format!("unknown image format: .{}", ext).into());
        };

        _ = image::load_from_memory_with_format(&bytes, format)?;

        Ok(Message::Image(name.to_owned(), ext.to_owned(), bytes))
    }

    /// deserialize a new message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Message, Box<dyn Error>> {
        // casting to dyn Error so we do not bind to specific implementation
        let message = bincode::deserialize(bytes)?;
        Ok(message)
    }

    /// encode message into bytes
    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // casting to dyn Error so we do not bind to specific implementation
        let bytes = bincode::serialize(self)?;
        Ok(bytes)
    }
}
