use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use futures::{SinkExt, StreamExt};
use prost::bytes::{Bytes, BytesMut};
use prost::Message;
use prost_types::Any;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::proto::sockets::transport::MessageTransport;
use crate::server_socket::message_handler::MessageHandler;

pub mod message_handler;

pub struct SocketOptions {
    pub bind_address: String,
    pub port: u16,
}

impl SocketOptions {
    pub fn new(bind_address: String, port: u16) -> SocketOptions {
        SocketOptions {
            bind_address,
            port,
        }
    }

    pub fn with_port(port: u16) -> SocketOptions {
        SocketOptions {
            bind_address: String::from("127.0.0.1"),
            port,
        }
    }

    fn get_socket_address(&self) -> Result<SocketAddr, <SocketAddr as FromStr>::Err> {
        let mut addr = self.bind_address.clone();
        addr.push(':');
        addr.push_str(&*self.port.to_string());
        addr.parse()
    }
}

pub struct ServerSocket {
    options: SocketOptions,
    pub message_handler: Arc<Mutex<MessageHandler>>
}

impl ServerSocket {
    pub async fn start(options: SocketOptions) -> ServerSocket {
        let server_socket = ServerSocket {
            options,
            message_handler: Arc::new(Mutex::new(MessageHandler::new()))
        };

        let addr = server_socket.options.get_socket_address().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let mut channel = ProtoChannel::new(stream, &server_socket.message_handler);

            tokio::spawn(async move {
                let _ = channel.process_stream().await; // TODO: handle channel connect err
            });
        }
    }
}

pub struct ProtoChannel {
    transport: Framed<TcpStream, LengthDelimitedCodec>,
    message_handler: Arc<Mutex<MessageHandler>>
}

impl ProtoChannel {
    pub fn new(tcp_stream: TcpStream, message_handler: &Arc<Mutex<MessageHandler>>) -> ProtoChannel {
        ProtoChannel {
            transport: Framed::new(tcp_stream, LengthDelimitedCodec::new()),
            message_handler: Arc::clone(message_handler)
        }
    }

    /// this function is blocking and continuously reads frames from the channel
    async fn process_stream(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            while let Some(frame) = self.transport.next().await {
                match frame {
                    Ok(data) => {
                        let result = self.process_frame(data, );
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error while processing packet; {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error while framing input stream; {}", e);
                    }
                }
            }
        }
    }

    /// this functions processes each individual frame from the channel
    fn process_frame(&self, frame: BytesMut) -> Result<(), Box<dyn Error>> {
        let transport_msg = MessageTransport::decode(frame)?;
        let any_msg = transport_msg.payload.unwrap();
        let acknowledgement_id = transport_msg.acknowledgement_id;
        let message_ctx = MessageContext::new(acknowledgement_id, any_msg.type_url, self);
        self.message_handler.lock().unwrap().handle_message(message_ctx, any_msg.value)?;
        Ok(())
    }

    /// send a message to the channel
    pub async fn send_message(&mut self, msg: Any) -> Result<(), Box<dyn Error>> {
        self.send_message_ack(msg, &None).await
    }

    /// send a message with an acknowledgement id
    pub async fn send_message_ack(&mut self, msg: Any, acknowledgement_id: &Option<String>) -> Result<(), Box<dyn Error>> {
        let mut transport_msg = MessageTransport::default();
        transport_msg.payload = Some(msg);
        transport_msg.acknowledgement_id = acknowledgement_id.clone();
        let data = Bytes::from(transport_msg.encode_to_vec());
        let _ = self.transport.send(data);
        Ok(())
    }
}

/// contains info of a message
pub struct MessageContext<'a> {
    acknowledgement_id: Option<String>,
    message_type: String,
    channel: &'a ProtoChannel,
}

impl<'a> MessageContext<'a> {
    pub fn new(acknowledgement_id: Option<String>, message_type: String, channel: &'a ProtoChannel) -> MessageContext<'a> {
        MessageContext {
            acknowledgement_id,
            message_type,
            channel
        }
    }

    /// returns true if an acknowledgement id is present and expects an acknowledgement
    pub fn needs_acknowledgement(&self) -> bool {
        self.acknowledgement_id.is_some()
    }
}
