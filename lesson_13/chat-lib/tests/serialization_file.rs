use std::error::Error;
use std::path::PathBuf;

use chat_lib::Message;

#[test]
fn file_serialization() -> Result<(), Box<dyn Error>> {
    let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_path.push("resources/test/calendar.ics");

    let msg = Message::new_file_message(test_file_path.to_str().unwrap())?;
    let encoded = msg.encode().unwrap();
    let decoded = Message::from_bytes(&encoded[..]).unwrap();

    assert!(matches!(msg, Message::File(_, _)));
    assert_eq!(msg, decoded);

    Ok(())
}

#[test]
fn missing_file() -> Result<(), Box<dyn Error>> {
    let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_path.push("resources/test/nonexistent.file");

    let msg = Message::new_file_message(test_file_path.to_str().unwrap());
    assert!(msg.is_err());

    Ok(())
}
