use keyring::Entry;
use std::env;
use std::sync::OnceLock;

pub struct CredentialManager{
    service: &'static str,
    username: String,
}

impl CredentialManager {
    pub fn new() -> Self {
        let username = env::var("USER").or_else(|_| env::var("USERNAME")).unwrap(); //you should always have a username
        Self { service : &"kilonova-cli", username }
    }

    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<CredentialManager> = OnceLock::new();
        INSTANCE.get_or_init(|| CredentialManager::new())
    }

    fn get_entry(&self) -> Option<Entry> {
        Entry::new(&self.username, self.service).ok()
    }
    pub fn get_token(&self) -> Option<String> {
        self.get_entry().and_then(|entry| entry.get_password().ok())
    }

    pub fn set_token(&self, token: &str) -> Result<(), String> {
        self.get_entry()
            .ok_or_else(|| "No entry found".to_owned())?
            .set_password(token)
            .map_err(|err| err.to_string())
    }
    pub fn delete_token(&self, target: &str) -> Result<(), String> {
        self.get_entry()
            .ok_or_else(|| "No entry found".to_owned())?
            .delete_credential()
            .map_err(|err| err.to_string())
    }
}
