use common::bytes::Bytes;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct UdpTuple {
    pub bytes: Bytes,
    pub peer: SocketAddr,
}

impl From<(Bytes, SocketAddr)> for UdpTuple {
    fn from(tuple: (Bytes, SocketAddr)) -> Self {
        Self {
            bytes: tuple.0,
            peer: tuple.1,
        }
    }
}

impl Into<(Bytes, SocketAddr)> for UdpTuple {
    fn into(self) -> (Bytes, SocketAddr) {
        (self.bytes, self.peer)
    }
}
