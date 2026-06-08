use std::any::Any;
use std::fmt::{Debug, Display, Formatter};

use bytes::{Bytes, BytesMut};

use crate::chunk::chunk_header::{ChunkHeader, CHUNK_HEADER_SIZE};
use crate::chunk::Chunk;

#[derive(Clone, Debug)]
pub struct ChunkUnknown {
    hdr: ChunkHeader,
    value: Bytes,
}

impl Display for ChunkUnknown {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkUnknown( {} {:?} )", self.hdr, self.value)
    }
}

impl Chunk for ChunkUnknown {
    #[tracing::instrument(level = "debug", skip(self))]
    fn header(&self) -> ChunkHeader {
        self.hdr.clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn check(&self) -> crate::error::Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn value_length(&self) -> usize {
        self.value.len()
    }

    #[tracing::instrument(level = "debug", skip(self, buf))]
    fn marshal_to(&self, buf: &mut BytesMut) -> crate::error::Result<usize> {
        self.header().marshal_to(buf)?;
        buf.extend(&self.value);
        Ok(buf.len())
    }

    #[tracing::instrument(level = "debug", skip(raw))]
    fn unmarshal(raw: &Bytes) -> crate::error::Result<Self>
    where
        Self: Sized,
    {
        let header = ChunkHeader::unmarshal(raw)?;
        let len = header.value_length();
        Ok(Self {
            hdr: header,
            value: raw.slice(CHUNK_HEADER_SIZE..CHUNK_HEADER_SIZE + len),
        })
    }
}
