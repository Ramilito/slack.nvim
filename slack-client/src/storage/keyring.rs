use anyhow::{Context, Result};
use keyring::Entry;

#[derive(Clone)]
pub struct Keyring {
    service: &'static str,
}

impl Keyring {
    pub fn new(service: &'static str) -> Self {
        Self { service }
    }

    fn entry(&self, profile: &str) -> Result<Entry> {
        Entry::new(self.service, profile).context("failed to create keyring entry")
    }

    pub fn store_token(&self, profile: &str, token: &str) -> Result<()> {
        self.entry(profile)?
            .set_password(token)
            .with_context(|| format!("failed to store Slack token in keyring (profile={profile})"))?;
        Ok(())
    }

    pub fn load_token(&self, profile: &str) -> Result<String> {
        self.entry(profile)?
            .get_password()
            .with_context(|| format!("failed to read Slack token from keyring (profile={profile})"))
    }
}
