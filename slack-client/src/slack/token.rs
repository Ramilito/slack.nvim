use anyhow::{bail, ensure, Result};
use slack_morphism::{SlackApiToken, SlackApiTokenValue};

fn redact(token: &str) -> String {
    let t = token.trim();
    if t.len() <= 10 {
        return "<redacted>".to_string();
    }
    format!("{}…{}", &t[..6], &t[t.len() - 4..])
}

/// “App token only” in the sense of “NO web/session tokens + cookies”.
/// We accept a single Slack token string (no `:`).
///
/// If you want to strictly enforce prefixes (xoxb-/xoxp-), do it here.
pub fn parse_app_token(raw: &str) -> Result<SlackApiToken> {
    let raw = raw.trim();
    ensure!(!raw.is_empty(), "Slack token is empty");

    // Old format was "<token>:<cookie>" - explicitly unsupported now.
    if raw.contains(':') {
        bail!(
            "web tokens/cookies are not supported anymore. Expected a single token string, got something like '{}'",
            redact(raw)
        );
    }

    // Optional guardrail: helps catch mistakes early.
    // Adjust if you truly want to allow other token types.
    if !(raw.starts_with("xoxb-") || raw.starts_with("xoxp-")) {
        bail!(
            "expected a Slack app OAuth token (xoxb-... or xoxp-...), got '{}'",
            redact(raw)
        );
    }

    let token_value: SlackApiTokenValue = raw.to_string().into();
    Ok(SlackApiToken::new(token_value))
}
