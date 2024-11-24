use crate::security::{ConfigurationError, CERTIFICATE_FILENAME, CERTIFICATE_PRIVATE_KEY_FILENAME};
use std::sync::Arc;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::TlsAcceptor;

pub fn setup_tls() -> Result<TlsAcceptor, ConfigurationError> {
    let certs = CertificateDer::pem_file_iter(CERTIFICATE_FILENAME)
        .map_err(|e| ConfigurationError::CertificateError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ConfigurationError::CertificateError(e.to_string()))?;
    let key = PrivateKeyDer::from_pem_file(CERTIFICATE_PRIVATE_KEY_FILENAME)
        .map_err(|e| ConfigurationError::PrivateKeyError(e.to_string()))?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| ConfigurationError::TlsConfigurationError(e.to_string()))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
