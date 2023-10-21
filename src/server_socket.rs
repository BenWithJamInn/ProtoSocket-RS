use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;

use futures::{SinkExt, StreamExt};
use prost::bytes::{Bytes, BytesMut};
use prost::Message;
use prost_types::Any;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::proto::sockets::transport::MessageTransport;

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
    // pub message_handler: MessageHandler<'a>
}

impl ServerSocket {
    pub async fn start(options: SocketOptions) -> ServerSocket {
        let server_socket = ServerSocket {
            options,
            // message_handler: MessageHandler::new()
        };

        let addr = server_socket.options.get_socket_address().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();

            tokio::spawn(async move {
                let _ = ServerSocket::process_stream(stream).await; // TODO: handle channel connect err
            });
        }
    }

    /// this function is blocking and continuously reads frames from the channel
    async fn process_stream(stream: TcpStream) -> Result<(), Box<dyn Error>> {
        let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
        let ctx = ChannelContext::new(&mut transport);
        loop {
            while let Some(frame) = ctx.transport.next().await {
                match frame {
                    Ok(data) => {
                        let result = ServerSocket::process_frame(data, &ctx);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error while processing packet; {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error while framing input stream; {}", e)
                    }
                }
            }
        }
    }

    /// this functions processes each individual frame from the channel
    fn process_frame(frame: BytesMut, ctx: &ChannelContext) -> Result<(), Box<dyn Error>> {
        let transport_msg = MessageTransport::decode(frame)?;
        let any_msg = transport_msg.payload.unwrap();
        let acknowledgement_id = transport_msg.acknowledgement_id;
        // let handle_result = self.message_handler.handle_message(any_msg, acknowledgement_id, ctx);
        // match handle_result {
        //     Ok(_) => {}
        //     Err(e) => {
        //         eprintln!("Error while handling packet {}; {}", any_msg.type_url, e);
        //     }
        // }
        Ok(())
    }
}

/// contains info of a channel and utility functions
pub struct ChannelContext<'a> {
    transport: &'a mut Framed<TcpStream, LengthDelimitedCodec>,
}

impl ChannelContext<'_> {
    pub fn new(transport: &mut Framed<TcpStream, LengthDelimitedCodec>) -> ChannelContext {
        ChannelContext {
            transport
        }
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
