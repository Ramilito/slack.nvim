use crate::slack::types::AuthSummary;
use slack_morphism::SlackApiToken;

#[derive(Clone, Debug)]
pub struct Profile {
    pub name: String,
}

#[derive(Clone)]
pub struct Token {
    pub token: SlackApiToken,
}

#[derive(Clone, Debug)]
pub struct Auth {
    pub summary: AuthSummary,
}
