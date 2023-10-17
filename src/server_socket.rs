use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::StreamExt;

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
    options: SocketOptions
}

impl ServerSocket {
    pub async fn start(options: SocketOptions) -> ServerSocket {
        let server_socket = ServerSocket {
            options
        };

        let addr = server_socket.options.get_socket_address().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();

            tokio::spawn(async move {
                if let Err(e) = process(stream).await {
                    println!("Failed to process connection")
                }
            });
        }
    }
}

async fn process(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
    loop {
        while let Some(frame) = transport.next().await {
            let data = frame.unwrap();
            println!("recieved data {:?}", data);
        }
    }
}
