use super::*;
use crate::message::packer::*;

// An AResource is an A Resource record.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AResource {
    pub a: [u8; 4],
}

impl fmt::Display for AResource {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dnsmessage.AResource{{A: {:?}}}", self.a)
    }
}

impl ResourceBody for AResource {
    #[tracing::instrument(level = "debug", skip(self))]
    fn real_type(&self) -> DnsType {
        DnsType::A
    }

    // pack appends the wire format of the AResource to msg.
    #[tracing::instrument(level = "debug", skip(self, msg, _compression, _compression_off))]
    fn pack(
        &self,
        msg: Vec<u8>,
        _compression: &mut Option<HashMap<String, usize>>,
        _compression_off: usize,
    ) -> Result<Vec<u8>> {
        Ok(pack_bytes(msg, &self.a))
    }

    #[tracing::instrument(level = "debug", skip(self, msg, off, _length))]
    fn unpack(&mut self, msg: &[u8], off: usize, _length: usize) -> Result<usize> {
        unpack_bytes(msg, off, &mut self.a)
    }
}
