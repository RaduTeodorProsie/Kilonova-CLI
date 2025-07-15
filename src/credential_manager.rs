use keyring::Entry;
use std::env;
use std::sync::OnceLock;

pub struct CredentialManager{
    username: String,
}

pub struct Token;
pub struct Cache;
pub trait IsService{
    fn service_name() -> &'static str;
}

impl IsService for Token{
    fn service_name() -> &'static str{
        "kilonova-cli-token"
    }
}

impl IsService for Cache{
    fn service_name() -> &'static str{
        "kilonova-cli-cache"
    }
}

impl CredentialManager {
    pub fn new() -> Self {
        let username = env::var("USER").or_else(|_| env::var("USERNAME")).unwrap(); //you should always have a username
        Self { username }
    }

    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<CredentialManager> = OnceLock::new();
        INSTANCE.get_or_init(|| CredentialManager::new())
    }

    fn get_entry<T : IsService>(&self) -> Option<Entry> {
        Entry::new(&self.username, T::service_name().to_string().as_ref()).ok()
    }
    pub fn get<T : IsService>(&self) -> Option<String> {
        self.get_entry::<T>().and_then(|entry| entry.get_password().ok())
    }

    pub fn set<T : IsService>(&self, value: &str) -> Result<(), String> {
        self.get_entry::<T>()
            .ok_or_else(|| "No entry found".to_owned())?
            .set_password(value)
            .map_err(|err| err.to_string())
    }
    pub fn delete<T : IsService>(&self) -> Result<(), String> {
        self.get_entry::<T>()
            .ok_or_else(|| "No entry found".to_owned())?
            .delete_credential()
            .map_err(|err| err.to_string())
    }
}
