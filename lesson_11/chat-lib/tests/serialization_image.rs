use std::error::Error;
use std::path::PathBuf;

use chat_lib::Message;

#[test]
fn image_serialization() -> Result<(), Box<dyn Error>> {
    let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_path.push("resources/test/rust.jpg");

    let msg = Message::new_image_message(test_file_path.to_str().unwrap())?;
    let encoded = msg.encode().unwrap();
    let decoded = Message::from_bytes(&encoded[..]).unwrap();

    assert!(matches!(msg, Message::Image(_, _, _)));
    assert_eq!(decoded, msg);

    Ok(())
}

#[test]
fn invalid_image_format() -> Result<(), Box<dyn Error>> {
    let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_path.push("resources/test/calendar.ics");

    let msg = Message::new_image_message(test_file_path.to_str().unwrap());
    assert!(msg.is_err());

    Ok(())
}
