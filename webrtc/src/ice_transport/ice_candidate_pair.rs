use std::fmt;

use crate::ice_transport::ice_candidate::*;

/// ICECandidatePair represents an ICE Candidate pair
///
/// ## Specifications
///
/// * [MDN]
///
/// [MDN]: https://developer.mozilla.org/en-US/docs/Web/API/RTCIceCandidatePair
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RTCIceCandidatePair {
    stats_id: String,
    pub local: RTCIceCandidate,
    pub remote: RTCIceCandidate,
}

impl fmt::Display for RTCIceCandidatePair {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(local) {} <-> (remote) {}", self.local, self.remote)
    }
}

impl RTCIceCandidatePair {
    #[tracing::instrument(level = "debug", skip(local_id, remote_id))]
    fn stats_id(local_id: &str, remote_id: &str) -> String {
        format!("{local_id}-{remote_id}")
    }

    #[tracing::instrument(level = "debug", skip(local, remote))]
    /// returns an initialized ICECandidatePair
    /// for the given pair of ICECandidate instances
    pub fn new(local: RTCIceCandidate, remote: RTCIceCandidate) -> Self {
        let stats_id = Self::stats_id(&local.stats_id, &remote.stats_id);
        RTCIceCandidatePair {
            stats_id,
            local,
            remote,
        }
    }
}
