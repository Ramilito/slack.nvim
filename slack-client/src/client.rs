use anyhow::Result;
use serde::Serialize;
use slack_morphism::prelude::*;

use crate::{active_context, http_client, runtime};

#[derive(Clone, Debug, Serialize)]
pub struct ConversationSummary {
    pub id: String,
    pub name: Option<String>,
    pub is_im: bool,
    pub is_private: bool,
    pub is_mpim: bool,
    pub is_archived: bool,
    pub num_members: Option<u64>,
}

pub fn list_conversations(limit: Option<u16>) -> Result<Vec<ConversationSummary>> {
    let ctx = active_context()?;
    let max = limit.unwrap_or(500) as usize;

    runtime().block_on(async move {
        let session = http_client().open_session(&ctx.token);

        let mut out: Vec<ConversationSummary> = Vec::new();
        let mut cursor: Option<SlackCursorId> = None;

        while out.len() < max {
            let mut req = SlackApiConversationsListRequest::new()
                .with_exclude_archived(true)
                .with_limit(std::cmp::min(200, (max - out.len()) as u16));

            if let Some(c) = cursor.clone() {
                req = req.with_cursor(c);
            }

            let resp = session.conversations_list(&req).await?;

            for ch in resp.channels.clone().into_iter() {
                let f = ch.flags.clone();
                out.push(ConversationSummary {
                    id: ch.id.to_string(),
                    name: ch.name.clone(),
                    is_im: f.is_im.unwrap_or(false),
                    is_private: f.is_private.unwrap_or(false),
                    is_mpim: f.is_mpim.unwrap_or(false),
                    is_archived: f.is_archived.unwrap_or(false),
                    num_members: ch.num_members,
                });

                if out.len() >= max {
                    break;
                }
            }

            cursor = resp.next_cursor().cloned();
            if cursor.is_none() {
                break;
            }
        }

        Ok(out)
    })
}

