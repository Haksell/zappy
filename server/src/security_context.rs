use regex::Regex;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct SecurityContext {
    users: HashMap<String, u64>,
}

impl SecurityContext {
    pub fn new(env: String) -> Result<SecurityContext, String> {
        let log_pass_regex =
            Regex::new(r"^[a-zA-Z0-9]+:[a-zA-Z0-9]+(?:,[a-zA-Z0-9]+:[a-zA-Z0-9]+)*$")
                .map_err(|_| "Invalid regex pattern")?;

        if !log_pass_regex.is_match(&env) {
            return Err("User credentials must be in format 'user1:pass1,user2:pass2'".to_string());
        }

        let users = env
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

    pub fn check_credentials(&self, username: &str, password: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let hashed_password = hasher.finish();
        self.users.contains_key(username) && self.users[username] == hashed_password
    }
}
