use std::net::Ipv4Addr;
use std::sync::Arc;

use super::*;
use crate::sync::RwLock;

/// Since UDP is connectionless, as a server, it doesn't know how to reply
/// simply using the `Write` method. So, to make it work, `disconnectedPacketConn`
/// will infer the last packet that it reads as the reply address for `Write`
pub struct DisconnectedPacketConn {
    raddr: RwLock<SocketAddr>,
    pconn: Arc<dyn Conn + Send + Sync>,
}

impl DisconnectedPacketConn {
    #[tracing::instrument(level = "debug", skip(conn))]
    pub fn new(conn: Arc<dyn Conn + Send + Sync>) -> Self {
        DisconnectedPacketConn {
            raddr: RwLock::new(SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0)),
            pconn: conn,
        }
    }
}

#[async_trait]
impl Conn for DisconnectedPacketConn {
    #[tracing::instrument(level = "debug", skip(self, addr))]
    async fn connect(&self, addr: SocketAddr) -> Result<()> {
        self.pconn.connect(addr).await
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let (n, addr) = self.pconn.recv_from(buf).await?;
        *self.raddr.write() = addr;
        Ok(n)
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.pconn.recv_from(buf).await
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    async fn send(&self, buf: &[u8]) -> Result<usize> {
        let addr = *self.raddr.read();
        self.pconn.send_to(buf, addr).await
    }

    #[tracing::instrument(level = "debug", skip(self, buf, target))]
    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize> {
        self.pconn.send_to(buf, target).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn local_addr(&self) -> Result<SocketAddr> {
        self.pconn.local_addr()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn remote_addr(&self) -> Option<SocketAddr> {
        let raddr = *self.raddr.read();
        Some(raddr)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn close(&self) -> Result<()> {
        self.pconn.close().await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}
