use anyhow::{Context, Result};

use crate::ecs::world::World;
use crate::slack::types::ConversationSummary;

#[tracing::instrument(skip(world))]
pub fn list_conversations(world: &mut World, limit: Option<u16>) -> Result<Vec<ConversationSummary>> {
    let token = world.active_token()?;
    let max = limit.unwrap_or(500) as usize;

    world
        .resources
        .runtime
        .block_on(world.resources.slack.list_conversations(&token, max))
        .context("Slack conversations.list failed")
}
