use super::*;
use crate::crypto::crypto_cbc::*;
use crate::prf::*;

#[derive(Clone)]
pub struct CipherSuiteAes256CbcSha {
    cbc: Option<CryptoCbc>,
    rsa: bool,
}

impl CipherSuiteAes256CbcSha {
    const PRF_MAC_LEN: usize = 20;
    const PRF_KEY_LEN: usize = 32;
    const PRF_IV_LEN: usize = 16;

    #[tracing::instrument(level = "debug", skip(rsa))]
    pub fn new(rsa: bool) -> Self {
        CipherSuiteAes256CbcSha { cbc: None, rsa }
    }
}

impl CipherSuite for CipherSuiteAes256CbcSha {
    #[tracing::instrument(level = "debug", skip(self))]
    fn to_string(&self) -> String {
        if self.rsa {
            "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA".to_owned()
        } else {
            "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA".to_owned()
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn id(&self) -> CipherSuiteId {
        if self.rsa {
            CipherSuiteId::Tls_Ecdhe_Rsa_With_Aes_256_Cbc_Sha
        } else {
            CipherSuiteId::Tls_Ecdhe_Ecdsa_With_Aes_256_Cbc_Sha
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn certificate_type(&self) -> ClientCertificateType {
        if self.rsa {
            ClientCertificateType::RsaSign
        } else {
            ClientCertificateType::EcdsaSign
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn hash_func(&self) -> CipherSuiteHash {
        CipherSuiteHash::Sha256
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_psk(&self) -> bool {
        false
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_initialized(&self) -> bool {
        self.cbc.is_some()
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
            CipherSuiteAes256CbcSha::PRF_MAC_LEN,
            CipherSuiteAes256CbcSha::PRF_KEY_LEN,
            CipherSuiteAes256CbcSha::PRF_IV_LEN,
            self.hash_func(),
        )?;

        if is_client {
            self.cbc = Some(CryptoCbc::new(
                &keys.client_write_key,
                &keys.client_mac_key,
                &keys.server_write_key,
                &keys.server_mac_key,
            )?);
        } else {
            self.cbc = Some(CryptoCbc::new(
                &keys.server_write_key,
                &keys.server_mac_key,
                &keys.client_write_key,
                &keys.client_mac_key,
            )?);
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pkt_rlh, raw))]
    fn encrypt(&self, pkt_rlh: &RecordLayerHeader, raw: &[u8]) -> Result<Vec<u8>> {
        let cg = self.cbc.as_ref().ok_or(Error::Other(
            "CipherSuite has not been initialized, unable to encrypt".to_owned(),
        ))?;
        cg.encrypt(pkt_rlh, raw)
    }

    #[tracing::instrument(level = "debug", skip(self, input))]
    fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>> {
        let cg = self.cbc.as_ref().ok_or(Error::Other(
            "CipherSuite has not been initialized, unable to decrypt".to_owned(),
        ))?;
        cg.decrypt(input)
    }
}
