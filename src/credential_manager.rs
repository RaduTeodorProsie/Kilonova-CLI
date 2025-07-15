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
        Entry::new(T::service_name().to_string().as_ref(), self.username.as_ref()).ok()
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

mod tests{
    use super::*;
    fn add_get_and_delete<T : IsService>(){
        const DUMMY_VAL: &str = "093c57be61f8785dab30e54632b9896b71ae8d41b930a2b525391b83d5941828";
        CredentialManager::global().set::<T>(DUMMY_VAL).expect("Error setting credential manager");
        let stored = CredentialManager::global().get::<T>().expect("Error getting the stored credential");
        assert_eq!(stored, DUMMY_VAL, "Stored credential doesn't match");
        CredentialManager::global().delete::<T>().expect("Error deleting credential");
    }
    #[test]
    fn test_services(){
        add_get_and_delete::<Token>();
        add_get_and_delete::<Cache>();
    }

}
