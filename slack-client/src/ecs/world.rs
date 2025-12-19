use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::ecs::components::{Auth, Profile, Token};
use crate::slack::api::SlackApi;
use crate::storage::keyring::Keyring;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EntityId(u64);

pub struct World {
    next_id: u64,

    // Index: profile name -> entity
    profile_index: HashMap<String, EntityId>,

    // ECS component storages
    profiles: HashMap<EntityId, Profile>,
    tokens: HashMap<EntityId, Token>,
    auths: HashMap<EntityId, Auth>,

    active_profile: Option<EntityId>,

    pub resources: Resources,
}

pub struct Resources {
    pub runtime: tokio::runtime::Runtime,
    pub slack: SlackApi,
    pub keyring: Keyring,
}

impl World {
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
        let slack = SlackApi::new().context("failed to create Slack API client")?;
        let keyring = Keyring::new("slack.nvim");

        Ok(Self {
            next_id: 1,
            profile_index: HashMap::new(),
            profiles: HashMap::new(),
            tokens: HashMap::new(),
            auths: HashMap::new(),
            active_profile: None,
            resources: Resources {
                runtime,
                slack,
                keyring,
            },
        })
    }

    pub fn get_or_spawn_profile(&mut self, profile: &str) -> EntityId {
        if let Some(&id) = self.profile_index.get(profile) {
            return id;
        }

        let id = EntityId(self.next_id);
        self.next_id += 1;

        self.profile_index.insert(profile.to_string(), id);
        self.profiles.insert(
            id,
            Profile {
                name: profile.to_string(),
            },
        );

        id
    }

    pub fn set_token(&mut self, entity: EntityId, token: slack_morphism::SlackApiToken) {
        self.tokens.insert(entity, Token { token });
    }

    pub fn set_auth(&mut self, entity: EntityId, summary: crate::slack::types::AuthSummary) {
        self.auths.insert(entity, Auth { summary });
    }

    pub fn set_active_profile(&mut self, entity: EntityId) {
        self.active_profile = Some(entity);
    }

    pub fn active_entity(&self) -> Result<EntityId> {
        self.active_profile
            .ok_or_else(|| anyhow!("slack.nvim: not initialized; call init_runtime(profile) first"))
    }

    pub fn active_token(&self) -> Result<slack_morphism::SlackApiToken> {
        let e = self.active_entity()?;
        let t = self
            .tokens
            .get(&e)
            .ok_or_else(|| anyhow!("slack.nvim: active profile has no token loaded"))?;
        Ok(t.token.clone())
    }

    pub fn maybe_cached_auth_for_profile(
        &self,
        profile: &str,
    ) -> Option<crate::slack::types::AuthSummary> {
        let e = self.profile_index.get(profile)?;
        self.auths.get(e).map(|a| a.summary.clone())
    }
}

// ---- Global World ----

static WORLD: OnceLock<Mutex<Option<World>>> = OnceLock::new();

fn world_cell() -> &'static Mutex<Option<World>> {
    WORLD.get_or_init(|| Mutex::new(None))
}

pub fn with_world<T>(f: impl FnOnce(&mut World) -> Result<T>) -> Result<T> {
    let cell = world_cell();

    let mut guard = cell.lock().expect("slack.nvim: world mutex poisoned");

    // Lazy, fallible init on first use
    if guard.is_none() {
        *guard = Some(World::new()?);
    }

    // Safe: ensured above
    f(guard.as_mut().unwrap())
}
