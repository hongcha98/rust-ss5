use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::{io, slice};
use std::io::ErrorKind;
use std::string::FromUtf8Error;
use bytes::{BufMut, BytesMut};

// socket5 https://www.ietf.org/rfc/rfc1928.txt
pub mod constant {
    pub const SOCKET5_VERSION: u8 = 0x05;
    pub const METHOD_NO_AUTHENTICATION: u8 = 0x00;
    pub const RSV: u8 = 0x00;
    pub const CMD_CONNECT: u8 = 0x01;
    pub const CMD_BIND: u8 = 0x02;
    pub const CMD_UDP: u8 = 0x03;
    pub const ATYP_IPV4: u8 = 0x01;
    pub const ATYP_DOMAINNAME: u8 = 0x03;
    pub const ATYP_IPV6: u8 = 0x04;
    pub const REP_SUCESS: u8 = 0x00;
    pub const REP_SERVER_FAIL: u8 = 0x01;
    pub const REP_CONN_NO: u8 = 0x02;
    pub const REP_NETWORK_NO: u8 = 0x03;
    pub const REP_HOST_NO: u8 = 0x04;
    pub const REP_CONN_REFUSED: u8 = 0x05;
    pub const REP_TTL_EXP: u8 = 0x06;
    pub const REP_CMD_NO: u8 = 0x07;
    pub const REP_ADDRESS_NO: u8 = 0x08;
    pub const REP_NO: u8 = 0x09;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    CONNECT,
    BIND,
    UDP,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    AddressTypeNo(u8),
    AddressDomainNo,
    VersionNo(u8),
    CommandNo(u8),
}


impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Error::AddressDomainNo
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    REP_SUCESS,
    REP_SERVER_FAIL,
    REP_CONN_NO,
    REP_NETWORK_NO,
    REP_HOST_NO,
    REP_CONN_REFUSED,
    REP_TTL_EXP,
    REP_CMD_NO,
    REP_ADDRESS_NO,
    REP_NO,
    OTHER(u8),
}

impl Error {
    pub fn to_reply(&self) -> Reply {
        Reply::from_u8(
            match self {
                Error::IoError(_) => constant::REP_SERVER_FAIL,
                Error::AddressTypeNo(_) => constant::REP_ADDRESS_NO,
                Error::AddressDomainNo => constant::REP_HOST_NO,
                Error::VersionNo(_) => constant::REP_NO,
                Error::CommandNo(_) => constant::REP_CMD_NO,
            }
        )
    }
}


impl Reply {
    pub fn from_u8(u: u8) -> Self {
        match u {
            constant::REP_SUCESS => Reply::REP_SUCESS,
            constant::REP_SERVER_FAIL => Reply::REP_SERVER_FAIL,
            constant::REP_CONN_NO => Reply::REP_CONN_NO,
            constant::REP_NETWORK_NO => Reply::REP_NETWORK_NO,
            constant::REP_HOST_NO => Reply::REP_HOST_NO,
            constant::REP_CONN_REFUSED => Reply::REP_CONN_REFUSED,
            constant::REP_TTL_EXP => Reply::REP_TTL_EXP,
            constant::REP_CMD_NO => Reply::REP_CMD_NO,
            constant::REP_ADDRESS_NO => Reply::REP_ADDRESS_NO,
            constant::REP_NO => Reply::REP_NO,
            _ => Reply::OTHER(u),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Reply::REP_SUCESS => constant::REP_SUCESS,
            Reply::REP_SERVER_FAIL => constant::REP_SERVER_FAIL,
            Reply::REP_CONN_NO => constant::REP_CONN_NO,
            Reply::REP_NETWORK_NO => constant::REP_NETWORK_NO,
            Reply::REP_HOST_NO => constant::REP_HOST_NO,
            Reply::REP_CONN_REFUSED => constant::REP_CONN_REFUSED,
            Reply::REP_TTL_EXP => constant::REP_TTL_EXP,
            Reply::REP_CMD_NO => constant::REP_CMD_NO,
            Reply::REP_ADDRESS_NO => constant::REP_ADDRESS_NO,
            Reply::REP_NO => constant::REP_NO,
            Reply::OTHER(u) => *u
        }
    }

    pub async fn from<T>(read: &mut T) -> Result<Self, Error>
        where T: AsyncRead + Unpin
    {
        let mut reply = [0; 2];
        read.read_exact(&mut reply).await?;
        Ok(Reply::from_u8(reply[1]))
    }

    pub async fn write<T>(&self, write: &mut T) -> Result<(), Error>
        where T: AsyncWrite + Unpin
    {
        write.write_all(&[constant::SOCKET5_VERSION, self.to_u8()]).await?;
        Ok(())
    }
}

impl Command {
    pub fn to_u8(&self) -> u8 {
        match self {
            Command::CONNECT => constant::CMD_CONNECT,
            Command::BIND => constant::CMD_BIND,
            Command::UDP => constant::CMD_UDP,
        }
    }

    pub fn from_u8(u: u8) -> Result<Self, Error> {
        match u {
            constant::CMD_CONNECT => Ok(Command::CONNECT),
            constant::CMD_BIND => Ok(Command::BIND),
            constant::CMD_UDP => Ok(Command::UDP),
            _ => Err(Error::CommandNo(u))
        }
    }

    pub async fn from<T>(read: &mut T) -> Result<Self, Error>
        where T: AsyncRead + Unpin
    {
        let mut head = [0; 3];
        read.read_exact(&mut head).await?;
        Command::from_u8(head[1])
    }

    pub async fn write<T>(&self, write: &mut T) -> Result<(), Error>
        where T: AsyncWrite + Unpin
    {
        write.write_all(&[constant::SOCKET5_VERSION, self.to_u8(), constant::RSV]).await?;
        Ok(())
    }
}


pub struct ShakeHands {
    pub methods: Vec<u8>,
}

impl ShakeHands {
    pub fn new(methods: Vec<u8>) -> Self {
        ShakeHands { methods }
    }

    pub async fn from<T>(read: &mut T) -> Result<Self, Error>
        where T: AsyncRead + Unpin
    {
        let mut head = [0; 2];
        read.read_exact(&mut head).await?;
        let methods = head[1];
        let mut methods = Vec::with_capacity(methods as usize);
        read.read_exact(&mut methods).await?;
        Ok(ShakeHands { methods })
    }

    pub async fn write<T>(&self, write: &mut T) -> Result<(), Error>
        where T: AsyncWrite + Unpin
    {
        let mut buf = BytesMut::with_capacity(2 + self.methods.len());
        buf.put_u8(constant::SOCKET5_VERSION);
        buf.put_u8(self.methods.len() as u8);
        buf.put_slice(&self.methods);
        write.write_all(&buf).await?;
        Ok(())
    }
}

pub enum Address {
    Address(SocketAddr),
    DomainName(String, u16),
}

impl Address {
    pub async fn from<T>(read: &mut T) -> Result<Self, Error>
        where T: AsyncRead + Unpin
    {
        let mut atyp = [0; 1];
        read.read_exact(&mut atyp).await?;
        Ok(match atyp[0] {
            0x01 => {
                let mut ipv4 = [0; 6];
                read.read_exact(&mut ipv4).await?;
                Address::Address(
                    SocketAddr::V4(
                        SocketAddrV4::new(
                            Ipv4Addr::new(ipv4[0], ipv4[1], ipv4[2], ipv4[3]),
                            88,
                        )))
            }
            0x03 => {
                let mut ipv6 = [0; 16];
                read.read_exact(&mut ipv6).await?;
                let ipv6: &[u16] = unsafe { slice::from_raw_parts(ipv6.as_ptr() as *const _, 9) };
                Address::Address(
                    SocketAddr::V6(
                        SocketAddrV6::new(
                            Ipv6Addr::new(ipv6[0], ipv6[1], ipv6[2], ipv6[3], ipv6[4], ipv6[5], ipv6[6], ipv6[7]),
                            ipv6[8],
                            0,
                            0,
                        )))
            }
            0x04 => {
                let mut domain_len = [0; 1];
                read.read_exact(&mut domain_len).await?;
                let domain_len = domain_len[0] as usize;
                let mut domain = Vec::with_capacity(domain_len + 2);
                read.read_exact(&mut domain).await?;
                let raw_port = &domain[domain_len..];
                let port = unsafe { u16::from_be(*(raw_port.as_ptr() as *const _)) };
                domain.truncate(domain_len);
                Address::DomainName(
                    match String::from_utf8(domain) {
                        Ok(name) => name,
                        Err(_) => {
                            return Err(Error::AddressDomainNo);
                        }
                    },
                    port,
                )
            }
            u => {
                return Err(Error::AddressTypeNo(u));
            }
        })
    }

    pub async fn write<T>(&self, _write: &mut T) -> Result<(), Error>
        where T: AsyncWrite + Unpin
    {
        Ok(())
    }
}

pub struct Proxy {
    command: Command,
    address: Address,
}

impl Proxy {
    pub fn new(command: Command, address: Address) -> Self {
        Proxy { command, address }
    }

    pub async fn from<T>(read: &mut T) -> Result<Self, Error>
        where T: AsyncRead + Unpin
    {
        Ok(Proxy {
            command: Command::from(read).await?,
            address: Address::from(read).await?,
        })
    }

    pub async fn write<T>(&self, write: &mut T) -> Result<(), Error>
        where T: AsyncWrite + Unpin
    {
        self.command.write(write).await?;
        self.address.write(write).await?;
        Ok(())
    }
}






