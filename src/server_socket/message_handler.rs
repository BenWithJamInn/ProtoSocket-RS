use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, write};
use std::sync::Arc;

use prost::Message;
use prost_types::Any;

use crate::server_socket::ChannelContext;

pub struct MessageHandler {
    listeners: HashMap<String, Arc<Box<dyn Fn(Vec<u8>, bool, MessageContext) -> Result<(), Box<dyn Error>>>>>,
}

impl MessageHandler {
    /// Creates new message handler
    pub fn new() -> MessageHandler {
        MessageHandler {
            listeners: HashMap::new(),
        }
    }

    /// Adds a new message listener
    pub fn add_listener<F>(&mut self, type_url: String, listener: F) where
        F: Fn(Vec<u8>, bool, MessageContext) -> Result<(), Box<dyn Error>> + 'static {
        self.listeners.insert(type_url, Arc::new(Box::new(listener)));
    }

    /// Handles a message
    pub fn handle_message<'a>(&self, message: Any, acknowledgement_id: Option<String>, ch_ctx: &'a mut ChannelContext<'a>) -> Result<(), Box<dyn Error>> {
        let arch_pointer = match self.listeners.get(&message.type_url) {
            None => return Err(Box::new(MissingListenerError::new(message.type_url))),
            Some(point) => point
        };
        let listener = Arc::clone(arch_pointer);
        let m_ctx = MessageContext {
            channel_context: ch_ctx,
            acknowledgement_id
        };
        // listener(message.value, message.type_url.is_empty(), m_ctx)
        Ok(())
    }
}

pub struct MessageContext<'a> {
    channel_context: &'a mut ChannelContext<'a>,
    acknowledgement_id: Option<String>
}

impl MessageContext<'_>  {
    pub async fn reply(&mut self, msg: Any) -> Result<(), Box<dyn Error>> {
        self.channel_context.send_message_ack(msg, &self.acknowledgement_id).await
    }

    pub async fn send_message(&mut self, msg: Any) -> Result<(), Box<dyn Error>> {
        self.channel_context.send_message(msg).await
    }
}

struct MissingListenerError {
    type_url: String
}

impl MissingListenerError {
    fn new(type_url: String) -> MissingListenerError {
        MissingListenerError {
            type_url
        }
    }
}

impl Display for MissingListenerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing listener for {} message!", self.type_url)
    }
}

impl Debug for MissingListenerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing listener for {} message!", self.type_url)
    }
}

impl Error for MissingListenerError {

}
