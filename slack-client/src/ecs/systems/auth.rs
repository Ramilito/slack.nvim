use anyhow::{Context, Result};

use crate::ecs::world::World;
use crate::slack::token::parse_app_token;

#[tracing::instrument(skip(world))]
pub fn register_profile(world: &mut World, profile: &str, token_raw: &str) -> Result<crate::slack::types::AuthSummary> {
    let token_raw = token_raw.trim();

    // Enforce “no web tokens”: reject old "<token>:<cookie>" format immediately.
    let token = parse_app_token(token_raw)
        .with_context(|| format!("invalid Slack token for profile '{profile}'"))?;

    // Validate token now (fast feedback)
    let auth = world
        .resources
        .runtime
        .block_on(world.resources.slack.auth_test(&token))
        .with_context(|| format!("Slack auth.test failed for profile '{profile}'"))?;

    // Store token ONLY
    world
        .resources
        .keyring
        .store_token(profile, token_raw)
        .with_context(|| format!("failed to store token in keyring (profile={profile})"))?;

    // ECS: keep entity up-to-date (but do NOT set active profile here)
    let e = world.get_or_spawn_profile(profile);
    world.set_token(e, token);
    world.set_auth(e, auth.clone());

    Ok(auth)
}

#[tracing::instrument(skip(world))]
pub fn init_profile(world: &mut World, profile: &str) -> Result<crate::slack::types::AuthSummary> {
    // Cheap path: if we already have cached auth for this profile, return it.
    // (Keeps behavior “simple” while avoiding unnecessary Slack calls.)
    if let Some(auth) = world.maybe_cached_auth_for_profile(profile) {
        // Also ensure active profile points here (init is meant to activate)
        let e = world.get_or_spawn_profile(profile);
        world.set_active_profile(e);
        return Ok(auth);
    }

    let token_raw = world
        .resources
        .keyring
        .load_token(profile)
        .with_context(|| format!("failed to read token from keyring (profile={profile})"))?;

    let token = parse_app_token(&token_raw)
        .with_context(|| format!("invalid Slack token stored in keyring (profile={profile})"))?;

    let auth = world
        .resources
        .runtime
        .block_on(world.resources.slack.auth_test(&token))
        .with_context(|| format!("Slack auth.test failed for profile '{profile}'"))?;

    let e = world.get_or_spawn_profile(profile);
    world.set_token(e, token);
    world.set_auth(e, auth.clone());
    world.set_active_profile(e);

    Ok(auth)
}
