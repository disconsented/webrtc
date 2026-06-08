use std::net::SocketAddr;

use ipnet::*;

use crate::error::*;

#[derive(Debug, Clone, Default)]
pub struct Interface {
    pub(crate) name: String,
    pub(crate) addrs: Vec<IpNet>,
}

impl Interface {
    #[tracing::instrument(level = "debug", skip(name, addrs))]
    pub fn new(name: String, addrs: Vec<IpNet>) -> Self {
        Interface { name, addrs }
    }

    #[tracing::instrument(level = "debug", skip(self, addr))]
    pub fn add_addr(&mut self, addr: IpNet) {
        self.addrs.push(addr);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn addrs(&self) -> &[IpNet] {
        &self.addrs
    }

    #[tracing::instrument(level = "debug", skip(addr, mask))]
    pub fn convert(addr: SocketAddr, mask: Option<SocketAddr>) -> Result<IpNet> {
        if let Some(mask) = mask {
            Ok(IpNet::with_netmask(addr.ip(), mask.ip()).map_err(|_| Error::ErrInvalidMask)?)
        } else {
            Ok(IpNet::new(addr.ip(), if addr.is_ipv4() { 32 } else { 128 })
                .expect("ipv4 should always work with prefix 32 and ipv6 with prefix 128"))
        }
    }
}
