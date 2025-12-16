// src/store.rs
use keyring::{Entry, Result as KeyringResult};

const SERVICE_NAME: &str = "slack-nvim";

pub fn save_token(profile: &str, token: &str) -> KeyringResult<()> {
    let entry = Entry::new(SERVICE_NAME, profile)?;
    entry.set_password(token)
}

pub fn load_token(profile: &str) -> KeyringResult<String> {
    let entry = Entry::new(SERVICE_NAME, profile)?;
    entry.get_password()
}

pub fn delete_token(profile: &str) -> KeyringResult<()> {
    let entry = Entry::new(SERVICE_NAME, profile)?;
    entry.delete_credential()
}
