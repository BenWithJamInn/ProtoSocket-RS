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
    use crate::server_socket::{ServerSocket, SocketOptions};

    static CONTROL_URL: String = String::from("proto.sockets.transport.TestControl");

    #[test]
    fn run_server() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut server = ServerSocket::start(SocketOptions::new(String::from("127.0.0.1"), 8080)).await;
                server.message_handler.add_listener(&CONTROL_URL, |data, ack, mut ctx| -> Result<(), Box<dyn Error>> {
                    let msg = TestControl::decode(data)?;
                    print!("Received a control test number: {} string: {}", msg.number?, msg.string?);
                    if ack {
                        let mut reply = TestControl::default();
                        reply.number = 64;
                        reply.string = String::from("Hello from ProtoSocket-RS!!");
                        let _ = ctx.reply(&reply);
                    }
                    Ok(())
                })
            })
    }
}
