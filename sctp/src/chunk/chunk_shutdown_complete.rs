use std::fmt;

use bytes::{Bytes, BytesMut};

use super::chunk_header::*;
use super::chunk_type::*;
use super::*;

///chunkShutdownComplete represents an SCTP Chunk of type chunkShutdownComplete
///
///0                   1                   2                   3
///0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///|   Type = 14   |Reserved     |T|      Length = 4               |
///+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[derive(Default, Debug, Clone)]
pub(crate) struct ChunkShutdownComplete;

/// makes chunkShutdownComplete printable
impl fmt::Display for ChunkShutdownComplete {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.header())
    }
}

impl Chunk for ChunkShutdownComplete {
    #[tracing::instrument(level = "debug", skip(self))]
    fn header(&self) -> ChunkHeader {
        ChunkHeader {
            typ: CT_SHUTDOWN_COMPLETE,
            flags: 0,
            value_length: self.value_length() as u16,
        }
    }

    #[tracing::instrument(level = "debug", skip(raw))]
    fn unmarshal(raw: &Bytes) -> Result<Self> {
        let header = ChunkHeader::unmarshal(raw)?;

        if header.typ != CT_SHUTDOWN_COMPLETE {
            return Err(Error::ErrChunkTypeNotShutdownComplete);
        }

        Ok(ChunkShutdownComplete {})
    }

    #[tracing::instrument(level = "debug", skip(self, writer))]
    fn marshal_to(&self, writer: &mut BytesMut) -> Result<usize> {
        self.header().marshal_to(writer)?;
        Ok(writer.len())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn check(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn value_length(&self) -> usize {
        0
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}
