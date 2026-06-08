use super::*;
use crate::client_certificate_type::ClientCertificateType;
use crate::crypto::crypto_ccm::{CryptoCcm, CryptoCcmTagLen};
use crate::prf::*;

#[derive(Clone)]
pub struct CipherSuiteAes128Ccm {
    ccm: Option<CryptoCcm>,
    client_certificate_type: ClientCertificateType,
    id: CipherSuiteId,
    psk: bool,
    crypto_ccm_tag_len: CryptoCcmTagLen,
}

impl CipherSuiteAes128Ccm {
    const PRF_MAC_LEN: usize = 0;
    const PRF_KEY_LEN: usize = 16;
    const PRF_IV_LEN: usize = 4;

    #[tracing::instrument(level = "debug", skip(client_certificate_type, id, psk, crypto_ccm_tag_len))]
    pub fn new(
        client_certificate_type: ClientCertificateType,
        id: CipherSuiteId,
        psk: bool,
        crypto_ccm_tag_len: CryptoCcmTagLen,
    ) -> Self {
        CipherSuiteAes128Ccm {
            ccm: None,
            client_certificate_type,
            id,
            psk,
            crypto_ccm_tag_len,
        }
    }
}

impl CipherSuite for CipherSuiteAes128Ccm {
    #[tracing::instrument(level = "debug", skip(self))]
    fn to_string(&self) -> String {
        format!("{}", self.id)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn id(&self) -> CipherSuiteId {
        self.id
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn certificate_type(&self) -> ClientCertificateType {
        self.client_certificate_type
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn hash_func(&self) -> CipherSuiteHash {
        CipherSuiteHash::Sha256
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_psk(&self) -> bool {
        self.psk
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_initialized(&self) -> bool {
        self.ccm.is_some()
    }

    #[tracing::instrument(level = "debug", skip(self, master_secret, client_random, server_random, is_client))]
    fn init(
        &mut self,
        master_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        is_client: bool,
    ) -> Result<()> {
        let keys = prf_encryption_keys(
            master_secret,
            client_random,
            server_random,
            CipherSuiteAes128Ccm::PRF_MAC_LEN,
            CipherSuiteAes128Ccm::PRF_KEY_LEN,
            CipherSuiteAes128Ccm::PRF_IV_LEN,
            self.hash_func(),
        )?;

        if is_client {
            self.ccm = Some(CryptoCcm::new(
                &self.crypto_ccm_tag_len,
                &keys.client_write_key,
                &keys.client_write_iv,
                &keys.server_write_key,
                &keys.server_write_iv,
            ));
        } else {
            self.ccm = Some(CryptoCcm::new(
                &self.crypto_ccm_tag_len,
                &keys.server_write_key,
                &keys.server_write_iv,
                &keys.client_write_key,
                &keys.client_write_iv,
            ));
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pkt_rlh, raw))]
    fn encrypt(&self, pkt_rlh: &RecordLayerHeader, raw: &[u8]) -> Result<Vec<u8>> {
        let ccm = self.ccm.as_ref().ok_or(Error::Other(
            "CipherSuite has not been initialized, unable to encrypt".to_owned(),
        ))?;
        ccm.encrypt(pkt_rlh, raw)
    }

    #[tracing::instrument(level = "debug", skip(self, input))]
    fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>> {
        let ccm = self.ccm.as_ref().ok_or(Error::Other(
            "CipherSuite has not been initialized, unable to decrypt".to_owned(),
        ))?;
        ccm.decrypt(input)
    }
}
