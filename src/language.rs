use super::credential_manager;
pub fn set_language(language: &str) {
    const ALLOWED: &[&str] = &[
        "cpp11",
        "cpp13",
        "cpp17",
        "cpp20",
        "python3",
        "rust",
        "go",
        "kotlin",
        "node.js",
        "outputonly",
        "pascal",
        "php",
    ];

    if ALLOWED.contains(&language) {
        let res = credential_manager::CredentialManager::global()
            .set::<credential_manager::Language>(&language);
        match res {
            Ok(()) => println!("Language set to {}", language),
            Err(e) => println!("Language not set, error: {}", e),
        }
    } else {
        println!(
            "unsupported language `{}`\n\
             allowed values are: {:?}",
            language, ALLOWED
        );
    }
}
