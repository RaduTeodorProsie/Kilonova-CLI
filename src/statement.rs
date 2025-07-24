use super::credential_manager;
pub fn set_language(name: &str) {
    const VALID: [&str; 2] = ["ro", "en"];
    if !VALID.contains(&name) {
        println!("{} is not a valid language name", name);
        println!("The valid languages are {:?}", VALID);
    } else {
        credential_manager::CredentialManager::global()
            .set::<credential_manager::StatementLanguage>(name)
            .expect("Couldn't set language");
        println!("Successfully set language to {}", name);
    }
}
