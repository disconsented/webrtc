// FIXME(regexident):
// Replace with `bytes::ExactSizeBuf` once merged:
// https://github.com/tokio-rs/bytes/pull/496

use bytes::buf::{Chain, Take};
use bytes::{Bytes, BytesMut};

/// A trait for buffers that know their exact length.
pub trait ExactSizeBuf {
    /// Returns the exact length of the buffer.
    fn len(&self) -> usize;

    /// Returns `true` if the buffer is empty.
    ///
    /// This method has a default implementation using `ExactSizeBuf::len()`,
    /// so you don't need to implement it yourself.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ExactSizeBuf for Bytes {
    #[tracing::instrument(level = "debug", skip(self))]
    #[inline]
    fn len(&self) -> usize {
        Bytes::len(self)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    #[inline]
    fn is_empty(&self) -> bool {
        Bytes::is_empty(self)
    }
}

impl ExactSizeBuf for BytesMut {
    #[tracing::instrument(level = "debug", skip(self))]
    #[inline]
    fn len(&self) -> usize {
        BytesMut::len(self)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    #[inline]
    fn is_empty(&self) -> bool {
        BytesMut::is_empty(self)
    }
}

impl ExactSizeBuf for [u8] {
    #[tracing::instrument(level = "debug", skip(self))]
    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    #[inline]
    fn is_empty(&self) -> bool {
        <[u8]>::is_empty(self)
    }
}

impl<T, U> ExactSizeBuf for Chain<T, U>
where
    T: ExactSizeBuf,
    U: ExactSizeBuf,
{
    #[tracing::instrument(level = "debug", skip(self))]
    fn len(&self) -> usize {
        let first_ref = self.first_ref();
        let last_ref = self.last_ref();

        first_ref.len() + last_ref.len()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_empty(&self) -> bool {
        let first_ref = self.first_ref();
        let last_ref = self.last_ref();

        first_ref.is_empty() && last_ref.is_empty()
    }
}

impl<T> ExactSizeBuf for Take<T>
where
    T: ExactSizeBuf,
{
    #[tracing::instrument(level = "debug", skip(self))]
    fn len(&self) -> usize {
        let inner_ref = self.get_ref();
        let limit = self.limit();

        limit.min(inner_ref.len())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_empty(&self) -> bool {
        let inner_ref = self.get_ref();
        let limit = self.limit();

        (limit == 0) || inner_ref.is_empty()
    }
}
