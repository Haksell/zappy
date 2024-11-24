use crate::security::{ConfigurationError, ADMIN_CREDENTIALS_ENV_VAR};
use dotenv::dotenv;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct SecurityContext {
    users: HashMap<String, u64>,
}

impl SecurityContext {
    pub fn from_env() -> Result<SecurityContext, ConfigurationError> {
        dotenv().ok();
        let credentials = env::var(ADMIN_CREDENTIALS_ENV_VAR)
            .map_err(|_| ConfigurationError::CredentialsVarIsNotSet)?;
        let log_pass_regex =
            Regex::new(r"^[a-zA-Z0-9]+:[a-zA-Z0-9]+(?:,[a-zA-Z0-9]+:[a-zA-Z0-9]+)*$").unwrap();

        if !log_pass_regex.is_match(&credentials) {
            Err(ConfigurationError::WrongCredentialsFormat)
        } else {
            let users = credentials
                .split(',')
                .filter_map(|chunk| {
                    let mut parts = chunk.split(':');
                    match (parts.next(), parts.next()) {
                        (Some(username), Some(password)) => {
                            let mut hasher = DefaultHasher::new();
                            password.hash(&mut hasher);
                            Some((username.to_string(), hasher.finish()))
                        }
                        _ => None,
                    }
                })
                .collect();

            Ok(SecurityContext { users })
        }
    }

    pub fn is_valid(&self, username: &str, password: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let hashed_password = hasher.finish();
        self.users.contains_key(username) && self.users[username] == hashed_password
    }
}
