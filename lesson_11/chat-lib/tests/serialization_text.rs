use chat_lib::Message;

#[test]
fn text_serialization() {
    let msg = Message::new_text_message("text message");
    let encoded = msg.encode().unwrap();
    let decoded = Message::from_bytes(&encoded[..]).unwrap();

    assert!(matches!(msg, Message::Text(_)));
    assert_eq!(msg, decoded);
}
