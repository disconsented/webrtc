use bytes::{Bytes, BytesMut};

use super::param_header::*;
use super::param_type::*;
use super::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct ParamRandom {
    pub(crate) random_data: Bytes,
}

impl fmt::Display for ParamRandom {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.header(), self.random_data)
    }
}

impl Param for ParamRandom {
    #[tracing::instrument(level = "debug", skip(self))]
    fn header(&self) -> ParamHeader {
        ParamHeader {
            typ: ParamType::Random,
            value_length: self.value_length() as u16,
        }
    }

    #[tracing::instrument(level = "debug", skip(raw))]
    fn unmarshal(raw: &Bytes) -> Result<Self> {
        let header = ParamHeader::unmarshal(raw)?;
        let random_data =
            raw.slice(PARAM_HEADER_LENGTH..PARAM_HEADER_LENGTH + header.value_length());
        Ok(ParamRandom { random_data })
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    fn marshal_to(&self, buf: &mut BytesMut) -> Result<usize> {
        self.header().marshal_to(buf)?;
        buf.extend(self.random_data.clone());
        Ok(buf.len())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn value_length(&self) -> usize {
        self.random_data.len()
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
