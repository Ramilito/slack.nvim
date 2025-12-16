// src/token.rs
use slack_morphism::prelude::*;

#[derive(Debug, Clone)]
pub struct SessionTokenParts {
    pub token: String,
    pub cookie: Option<String>,
}

/// Parse a wee-slack style token string:
/// - "xoxc-..."                     -> token only
/// - "xoxc-...:cookie"              -> token + cookie (treated as d=<cookie>)
/// - "xoxc-...:d=...;d-s=..."       -> token + full cookie header
pub fn parse_session_token(input: &str) -> SessionTokenParts {
    let input = input.trim();

    if let Some((token_raw, cookie_raw)) = input.split_once(':') {
        let token = token_raw.trim().to_string();
        let mut cookie = cookie_raw.trim().to_string();

        // If the cookie string doesn't contain '=', assume it's just the value of `d`
        if !cookie.is_empty() && !cookie.contains('=') {
            cookie = format!("d={cookie}");
        }

        SessionTokenParts {
            token,
            cookie: if cookie.is_empty() { None } else { Some(cookie) },
        }
    } else {
        SessionTokenParts {
            token: input.to_string(),
            cookie: None,
        }
    }
}

/// Build a Slack Morphism token from parsed parts.
pub fn build_slack_token(parts: &SessionTokenParts) -> SlackApiToken {
    let token_value: SlackApiTokenValue = parts.token.clone().into();
    let mut token = SlackApiToken::new(token_value);

    if let Some(cookie_str) = &parts.cookie {
        token = token.with_cookie(SlackApiCookieValue::from(cookie_str.as_str()));
    }

    token
}
