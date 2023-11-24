use chat_lib::{ChatMessageError, Message};

#[test]
fn message_errors() {
    let m = Message::new_image_message("something");

    assert!(m.is_err());

    let e = m.err().unwrap();
    let e = e.downcast_ref::<ChatMessageError>().unwrap();
    match e {
        ChatMessageError::FileReadError(_path) => {}
        _ => panic!("invalid error thrown"),
    }
}
