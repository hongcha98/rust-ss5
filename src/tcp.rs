use tokio::io::{AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;

use crate::config::ServerConfig;
use crate::socket5::{Command, Error, Proxy, Reply, ShakeHands};
use crate::socket5::constant::*;
use log::info;


pub struct TcpSocksClient {
    stream: TcpStream,
}

impl TcpSocksClient {
    pub fn new(stream: TcpStream) -> Self {
        TcpSocksClient {
            stream
        }
    }

    pub async fn server_connect(mut self, _config: ServerConfig) -> Result<(), Error> {
        let stream = &mut self.stream;
        let _hands = ShakeHands::from(stream).await?;
        stream.write_all(&[SOCKET5_VERSION, METHOD_NO_AUTHENTICATION]).await?;
        let proxy = Proxy::from(stream).await?;
        info!("{:?}", proxy);
        if proxy.command == Command::CONNECT {
            let mut proxy_stream = proxy.address.connect().await?;
            Reply::RepSuccess.write(stream).await?;
            proxy.address.write(stream).await?;
            copy_bidirectional(stream, &mut proxy_stream).await?;
        }
        Ok(())
    }


    // pub async fn client_connect<A: ToSocketAddrs>(addr: A, proxy: Proxy) -> Result<Self, Error> {
    //     let mut stream = TcpStream::connect(addr).await?;
    //     ShakeHands::new(vec![METHOD_NO_AUTHENTICATION]).write(&mut stream).await?;
    //     if let Reply::OTHER(u) = Reply::from(&mut stream).await? {
    //         if u != METHOD_NO_AUTHENTICATION {
    //             return Err(Error::AddressDomainNo);
    //         }
    //     };
    //     proxy.write(&mut stream).await?;
    //     Ok(TcpSocksClient { stream })
    // }
}


#[cfg(test)]
mod tests {
    use std::net::SocketAddrV4;

    use crate::socket5::{Address, Command, Proxy};
    use crate::tcp::TcpSocksClient;

    #[tokio::test]
    async fn client_connect_test() {
        let client = TcpSocksClient::client_connect(
            "127.0.0.1:9999",
            Proxy::new(
                Command::CONNECT,
                Address::DomainName("baidu.com".to_string(), 80),
            ),
        ).await.unwrap();
    }
}