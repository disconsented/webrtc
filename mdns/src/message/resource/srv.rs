use super::*;
use crate::error::Result;
use crate::message::name::*;
use crate::message::packer::*;

// An SRVResource is an SRV Resource record.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SrvResource {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: Name, // Not compressed as per RFC 2782.
}

impl fmt::Display for SrvResource {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "dnsmessage.SRVResource{{priority: {}, weight: {}, port: {}, target: {}}}",
            self.priority, self.weight, self.port, self.target
        )
    }
}

impl ResourceBody for SrvResource {
    #[tracing::instrument(level = "debug", skip(self))]
    fn real_type(&self) -> DnsType {
        DnsType::Srv
    }

    // pack appends the wire format of the SRVResource to msg.
    #[tracing::instrument(level = "debug", skip(self, msg, _compression, compression_off))]
    fn pack(
        &self,
        mut msg: Vec<u8>,
        _compression: &mut Option<HashMap<String, usize>>,
        compression_off: usize,
    ) -> Result<Vec<u8>> {
        msg = pack_uint16(msg, self.priority);
        msg = pack_uint16(msg, self.weight);
        msg = pack_uint16(msg, self.port);
        msg = self.target.pack(msg, &mut None, compression_off)?;
        Ok(msg)
    }

    #[tracing::instrument(level = "debug", skip(self, msg, off, _length))]
    fn unpack(&mut self, msg: &[u8], off: usize, _length: usize) -> Result<usize> {
        let (priority, off) = unpack_uint16(msg, off)?;
        self.priority = priority;

        let (weight, off) = unpack_uint16(msg, off)?;
        self.weight = weight;

        let (port, off) = unpack_uint16(msg, off)?;
        self.port = port;

        let off = self
            .target
            .unpack_compressed(msg, off, false /* allowCompression */)?;

        Ok(off)
    }
}
