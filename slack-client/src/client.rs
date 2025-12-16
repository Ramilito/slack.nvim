use slack_morphism::{
    errors::SlackClientError,
    hyper_tokio::{SlackClientHyperConnector, SlackHyperClient},
    SlackApiToken,
};
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static CLIENT: OnceLock<SlackHyperClient> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime")
    })
}

fn client() -> &'static SlackHyperClient {
    CLIENT.get_or_init(|| {
        // new() exists for the default HTTPS specialization :contentReference[oaicite:3]{index=3}
        let connector =
            SlackClientHyperConnector::new().expect("failed to create SlackClientHyperConnector");
        SlackHyperClient::new(connector)
    })
}

#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub user_id: String,
    pub team_id: String,
    pub user: Option<String>,
    pub team: String,
    pub bot_id: Option<String>,
    pub url: String,
}

pub fn test_auth(token: SlackApiToken) -> Result<AuthInfo, SlackClientError> {
    runtime().block_on(async move {
        let session = client().open_session(&token);
        let resp = session.auth_test().await?;

        Ok(AuthInfo {
            user_id: resp.user_id.to_string(),
            team_id: resp.team_id.to_string(),
            user: resp.user.clone(),
            team: resp.team.clone(),
            bot_id: resp.bot_id.map(|b| b.to_string()),
            url: resp.url.0.to_string(),
        })
    })
}
