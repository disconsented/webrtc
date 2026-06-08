#[cfg(test)]
mod handshake_message_server_hello_done_test;

use std::io::{Read, Write};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeMessageServerHelloDone;

impl HandshakeMessageServerHelloDone {
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn handshake_type(&self) -> HandshakeType {
        HandshakeType::ServerHelloDone
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn size(&self) -> usize {
        0
    }

    #[tracing::instrument(level = "debug", skip(self, _writer))]
    pub fn marshal<W: Write>(&self, _writer: &mut W) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(_reader))]
    pub fn unmarshal<R: Read>(_reader: &mut R) -> Result<Self> {
        Ok(HandshakeMessageServerHelloDone {})
    }
}
