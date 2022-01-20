use std::net::Ipv4Addr;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;
use rust_ss5::config::ServerConfig;
use rust_ss5::tcp::TcpSocksClient;
use log::{LevelFilter, info};

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), 9999)).await.unwrap();
    info!("start socks5 server, port : {}",9999);
    loop {
        match listener.accept().await {
            Ok((stream, address)) => {
                info!("received request address : {:?}",address);
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