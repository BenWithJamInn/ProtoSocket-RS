extern crate core;

pub mod server_socket;

pub mod proto {
    pub mod sockets {
        pub mod transport {
            include!(concat!(env!("OUT_DIR"), "/proto.sockets.transport.rs"));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prost::Message;

    use crate::proto::sockets::transport::TestControl;
    use crate::server_socket::{MessageContext, ServerSocket, SocketOptions};

    #[test]
    fn run_server() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let server = ServerSocket::start(SocketOptions::new(String::from("127.0.0.1"), 8080)).await;
                server.message_handler.lock().unwrap().add_listener("proto.sockets.transport.TestControl", handle_test_control);
            })
    }

    fn handle_test_control(message_ctx: MessageContext, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let control = TestControl::decode(message.as_slice())?;
        println!("{:?}", control);
        Ok(())
    }
}
