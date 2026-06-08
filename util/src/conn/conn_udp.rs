use tokio::net::UdpSocket;

use super::*;

#[async_trait]
impl Conn for UdpSocket {
    #[tracing::instrument(level = "debug", skip(self, addr))]
    async fn connect(&self, addr: SocketAddr) -> Result<()> {
        Ok(self.connect(addr).await?)
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.recv(buf).await?)
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        Ok(self.recv_from(buf).await?)
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    async fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(self.send(buf).await?)
    }

    #[tracing::instrument(level = "debug", skip(self, buf, target))]
    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize> {
        Ok(self.send_to(buf, target).await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.local_addr()?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn close(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}
