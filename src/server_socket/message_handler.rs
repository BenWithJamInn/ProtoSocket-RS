use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

use crate::server_socket::MessageContext;

type Callback = fn(MessageContext, Vec<u8>) -> Result<(), Box<dyn Error>>;

pub struct MessageHandler {
    listeners: Mutex<HashMap<String, Callback>>,
}

impl MessageHandler {
    /// Creates new message handler
    pub fn new() -> MessageHandler {
        MessageHandler {
            listeners: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_listener(&mut self, message_type: &str, callback: Callback) {
        self.listeners.lock().unwrap().insert(message_type.to_string(), callback);
    }

    pub fn remove_listener(&mut self, message_type: String) {
        self.listeners.lock().unwrap().remove(&message_type);
    }

    pub fn handle_message(&self, message_ctx: MessageContext, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
        match self.listeners.lock().unwrap().get(message_ctx.message_type.as_str()) {
            None => {
                Ok(())
            }
            Some(callback) => {
                callback(message_ctx, message)
            }
        }
    }
}