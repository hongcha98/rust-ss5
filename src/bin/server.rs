use std::net::Ipv4Addr;
use tokio::net::TcpListener;
use rust_ss5::config::ServerConfig;
use rust_ss5::tcp::TcpSocksClient;


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), 9999)).await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _address)) => {
                tokio::spawn(TcpSocksClient::new(stream).server_connect(ServerConfig {
                    port: 0,
                    password: "".to_string(),
                    encrypt: "".to_string(),
                }));
            }
            Err(_) => {
                continue;
            }
        };
    };
}