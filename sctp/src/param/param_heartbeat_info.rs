use bytes::{Bytes, BytesMut};

use super::param_header::*;
use super::param_type::*;
use super::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct ParamHeartbeatInfo {
    pub(crate) heartbeat_information: Bytes,
}

impl fmt::Display for ParamHeartbeatInfo {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.header(), self.heartbeat_information)
    }
}

impl Param for ParamHeartbeatInfo {
    #[tracing::instrument(level = "debug", skip(self))]
    fn header(&self) -> ParamHeader {
        ParamHeader {
            typ: ParamType::HeartbeatInfo,
            value_length: self.value_length() as u16,
        }
    }

    #[tracing::instrument(level = "debug", skip(raw))]
    fn unmarshal(raw: &Bytes) -> Result<Self> {
        let header = ParamHeader::unmarshal(raw)?;
        let heartbeat_information =
            raw.slice(PARAM_HEADER_LENGTH..PARAM_HEADER_LENGTH + header.value_length());
        Ok(ParamHeartbeatInfo {
            heartbeat_information,
        })
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    fn marshal_to(&self, buf: &mut BytesMut) -> Result<usize> {
        self.header().marshal_to(buf)?;
        buf.extend(self.heartbeat_information.clone());
        Ok(buf.len())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn value_length(&self) -> usize {
        self.heartbeat_information.len()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn clone_to(&self) -> Box<dyn Param + Send + Sync> {
        Box::new(self.clone())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}
