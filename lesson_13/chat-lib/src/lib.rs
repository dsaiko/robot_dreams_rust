use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ChatMessageError {
    #[error("Unable to read file `{0}`")]
    FileReadError(String),
    #[error("Invalid image format `{0}`")]
    InvalidImageFormat(String),
    #[error("Invalid image `{0}`")]
    InvalidImage(String),
    #[error("Error: `{0}`")]
    OtherError(String),
}

impl Message {
    /// construct a new message from a file
    pub fn new_file_message(file: &str) -> Result<Message> {
        let bytes = fs::read(file).context(ChatMessageError::FileReadError(file.to_owned()))?;
        Ok(Message::File(file.to_string(), bytes))
    }

    /// construct a new text message
    pub fn new_text_message(text: &str) -> Message {
        Message::Text(text.to_owned())
    }

    /// construct a new message from image file
    pub fn new_image_message(file: &str) -> Result<Message> {
        // read image as bytes
        let bytes = fs::read(file).context(ChatMessageError::FileReadError(file.to_owned()))?;
        let Some(name) = Path::new(file).file_stem().and_then(OsStr::to_str) else {
            return Err(ChatMessageError::OtherError("Unable to get file name".to_owned()).into());
        };

        let Some(ext) = Path::new(file).extension().and_then(OsStr::to_str) else {
            return Err(
                ChatMessageError::OtherError("Unable to get file extension".to_owned()).into(),
            );
        };

        // validate image format
        let Some(format) = ImageFormat::from_extension(ext) else {
            return Err(ChatMessageError::InvalidImageFormat(ext.to_string()).into());
        };

        // validate image
        _ = image::load_from_memory_with_format(&bytes, format)
            .context(ChatMessageError::InvalidImage(file.to_owned()));

        Ok(Message::Image(name.to_owned(), ext.to_owned(), bytes))
    }

    /// deserialize a new message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Message> {
        let message = bincode::deserialize(bytes)?;
        Ok(message)
    }

    /// encode message into bytes
    pub fn encode(&self) -> Result<Vec<u8>> {
        // casting to dyn Error so we do not bind to specific implementation
        let bytes = bincode::serialize(self)?;
        Ok(bytes)
    }
}
