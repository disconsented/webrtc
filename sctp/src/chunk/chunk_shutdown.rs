use std::fmt;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::chunk_header::*;
use super::chunk_type::*;
use super::*;

///chunkShutdown represents an SCTP Chunk of type chunkShutdown
///
///0                   1                   2                   3
///0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///|   Type = 7    | Chunk  Flags  |      Length = 8               |
///+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///|                      Cumulative TSN Ack                       |
///+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[derive(Default, Debug, Clone)]
pub(crate) struct ChunkShutdown {
    pub(crate) cumulative_tsn_ack: u32,
}

pub(crate) const CUMULATIVE_TSN_ACK_LENGTH: usize = 4;

/// makes chunkShutdown printable
impl fmt::Display for ChunkShutdown {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.header())
    }
}

impl Chunk for ChunkShutdown {
    #[tracing::instrument(level = "debug", skip(self))]
    fn header(&self) -> ChunkHeader {
        ChunkHeader {
            typ: CT_SHUTDOWN,
            flags: 0,
            value_length: self.value_length() as u16,
        }
    }

    #[tracing::instrument(level = "debug", skip(raw))]
    fn unmarshal(raw: &Bytes) -> Result<Self> {
        let header = ChunkHeader::unmarshal(raw)?;

        if header.typ != CT_SHUTDOWN {
            return Err(Error::ErrChunkTypeNotShutdown);
        }

        if raw.len() != CHUNK_HEADER_SIZE + CUMULATIVE_TSN_ACK_LENGTH {
            return Err(Error::ErrInvalidChunkSize);
        }

        let reader = &mut raw.slice(CHUNK_HEADER_SIZE..CHUNK_HEADER_SIZE + header.value_length());

        let cumulative_tsn_ack = reader.get_u32();

        Ok(ChunkShutdown { cumulative_tsn_ack })
    }

    #[tracing::instrument(level = "debug", skip(self, writer))]
    fn marshal_to(&self, writer: &mut BytesMut) -> Result<usize> {
        self.header().marshal_to(writer)?;
        writer.put_u32(self.cumulative_tsn_ack);
        Ok(writer.len())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn check(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn value_length(&self) -> usize {
        CUMULATIVE_TSN_ACK_LENGTH
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}
