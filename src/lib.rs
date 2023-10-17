pub mod server_socket;

#[cfg(test)]
mod tests {
    use crate::server_socket::{ServerSocket, SocketOptions};

    #[test]
    fn run_server() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let server = ServerSocket::start(SocketOptions::new(String::from("127.0.0.1"), 8080)).await;
            })
    }
}
