use anyhow::{Context, Result};
use slack_morphism::prelude::*;
use std::sync::Arc;

use crate::slack::types::{AuthSummary, ConversationSummary};

#[derive(Clone)]
pub struct SlackApi {
    http: Arc<SlackHyperClient>,
}

impl SlackApi {
    pub fn new() -> Result<Self> {
        let connector =
            SlackClientHyperConnector::new().context("failed to create Slack hyper connector")?;
        Ok(Self {
            http: Arc::new(SlackHyperClient::new(connector)),
        })
    }

    pub async fn auth_test(&self, token: &SlackApiToken) -> Result<AuthSummary> {
        let session = self.http.open_session(token);
        let resp = session
            .auth_test()
            .await
            .context("Slack API auth.test failed")?;
        Ok(AuthSummary::from_auth_test(&resp))
    }

    pub async fn list_conversations(
        &self,
        token: &SlackApiToken,
        max: usize,
    ) -> Result<Vec<ConversationSummary>> {
        let session = self.http.open_session(token);

        let mut out: Vec<ConversationSummary> = Vec::new();
        let mut cursor: Option<SlackCursorId> = None;

        while out.len() < max {
            let mut req = SlackApiConversationsListRequest::new()
                .with_exclude_archived(true)
                .with_limit(std::cmp::min(200, (max - out.len()) as u16));

            if let Some(c) = cursor.clone() {
                req = req.with_cursor(c);
            }

            let resp = session
                .conversations_list(&req)
                .await
                .context("Slack API conversations.list failed")?;

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
    }
}
