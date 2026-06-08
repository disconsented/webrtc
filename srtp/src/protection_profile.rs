/// ProtectionProfile specifies Cipher and AuthTag details, similar to TLS cipher suite
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProtectionProfile {
    #[default]
    Aes128CmHmacSha1_80 = 0x0001,
    Aes128CmHmacSha1_32 = 0x0002,
    Aes256CmHmacSha1_80 = 0x0003,
    Aes256CmHmacSha1_32 = 0x0004,
    AeadAes128Gcm = 0x0007,
    AeadAes256Gcm = 0x0008,
}

impl ProtectionProfile {
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn key_len(&self) -> usize {
        match *self {
            ProtectionProfile::Aes128CmHmacSha1_32
            | ProtectionProfile::Aes128CmHmacSha1_80
            | ProtectionProfile::AeadAes128Gcm => 16,
            ProtectionProfile::Aes256CmHmacSha1_32 | ProtectionProfile::Aes256CmHmacSha1_80 => 32,
            ProtectionProfile::AeadAes256Gcm => 32,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn salt_len(&self) -> usize {
        match *self {
            ProtectionProfile::Aes128CmHmacSha1_32
            | ProtectionProfile::Aes128CmHmacSha1_80
            | ProtectionProfile::Aes256CmHmacSha1_32
            | ProtectionProfile::Aes256CmHmacSha1_80 => 14,
            ProtectionProfile::AeadAes128Gcm | ProtectionProfile::AeadAes256Gcm => 12,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn rtp_auth_tag_len(&self) -> usize {
        match *self {
            ProtectionProfile::Aes128CmHmacSha1_80 | ProtectionProfile::Aes256CmHmacSha1_80 => 10,
            ProtectionProfile::Aes128CmHmacSha1_32 | ProtectionProfile::Aes256CmHmacSha1_32 => 4,
            ProtectionProfile::AeadAes128Gcm | ProtectionProfile::AeadAes256Gcm => 0,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn rtcp_auth_tag_len(&self) -> usize {
        match *self {
            ProtectionProfile::Aes128CmHmacSha1_80
            | ProtectionProfile::Aes128CmHmacSha1_32
            | ProtectionProfile::Aes256CmHmacSha1_80
            | ProtectionProfile::Aes256CmHmacSha1_32 => 10,
            ProtectionProfile::AeadAes128Gcm | ProtectionProfile::AeadAes256Gcm => 0,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn aead_auth_tag_len(&self) -> usize {
        match *self {
            ProtectionProfile::Aes128CmHmacSha1_80
            | ProtectionProfile::Aes128CmHmacSha1_32
            | ProtectionProfile::Aes256CmHmacSha1_80
            | ProtectionProfile::Aes256CmHmacSha1_32 => 0,
            ProtectionProfile::AeadAes128Gcm | ProtectionProfile::AeadAes256Gcm => 16,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn auth_key_len(&self) -> usize {
        match *self {
            ProtectionProfile::Aes128CmHmacSha1_80
            | ProtectionProfile::Aes128CmHmacSha1_32
            | ProtectionProfile::Aes256CmHmacSha1_80
            | ProtectionProfile::Aes256CmHmacSha1_32 => 20,
            ProtectionProfile::AeadAes128Gcm | ProtectionProfile::AeadAes256Gcm => 0,
        }
    }
}
