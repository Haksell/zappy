use std::error::Error;
use std::fmt::{Debug, Formatter};

pub mod security_context;
pub mod tls;

const CERTIFICATE_FILENAME: &str = "cert.pem";
const CERTIFICATE_PRIVATE_KEY_FILENAME: &str = "key.pem";
const ADMIN_CREDENTIALS_ENV_VAR: &str = "ADMIN_CREDENTIALS";

pub enum ConfigurationError {
    CertificateError(String),
    PrivateKeyError(String),
    TlsConfigurationError(String),
    CredentialsVarIsNotSet,
    WrongCredentialsFormat,
}

impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            ConfigurationError::CertificateError(e) => format!("{CERTIFICATE_FILENAME}: {e}"),
            ConfigurationError::PrivateKeyError(e) => {
                format!("{CERTIFICATE_PRIVATE_KEY_FILENAME}: {e}")
            }
            ConfigurationError::TlsConfigurationError(e) => format!("Certificate issue: {e}"),
            ConfigurationError::CredentialsVarIsNotSet => {
                format!("Expecting {ADMIN_CREDENTIALS_ENV_VAR} to be set in .env file.")
            }
            ConfigurationError::WrongCredentialsFormat => {
                "User credentials must be in format 'user1:pass1,user2:pass2'".to_string()
            }
        };
        write!(f, "{}", err)
    }
}

impl Debug for ConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_string().fmt(f)
    }
}

impl Error for ConfigurationError {}
