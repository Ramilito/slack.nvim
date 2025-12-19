use serde::Serialize;
use slack_morphism::api::SlackApiAuthTestResponse;

#[derive(Clone, Debug, Serialize)]
pub struct AuthSummary {
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub team: Option<String>,
    pub user: Option<String>,
    pub url: Option<String>,
}

impl AuthSummary {
    pub(crate) fn from_auth_test(r: &SlackApiAuthTestResponse) -> Self {
        Self {
            team_id: Some(r.team_id.to_string()),
            user_id: Some(r.user_id.to_string()),
            team: Some(r.team.clone()),
            user: r.user.clone(),
            url: Some(r.url.0.to_string()),
        }
    }
}

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
