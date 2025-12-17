use anyhow::{anyhow, Context, Result};
use keyring::Entry;
use mlua::prelude::*;
use mlua::Lua;
use serde::Serialize;
use slack_morphism::{
    api::SlackApiAuthTestResponse, prelude::SlackClientHyperConnector, prelude::SlackHyperClient,
    SlackApiCookieValue, SlackApiToken, SlackApiTokenValue,
};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::runtime::Runtime;

mod client;

cfg_if::cfg_if! {
    if #[cfg(feature = "telemetry")] {
        use slack_telemetry as logging;
    } else {
        mod log;
        use log as logging;
    }
}

const KEYRING_SERVICE: &str = "slack.nvim";

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static HTTP_CLIENT: OnceLock<Arc<SlackHyperClient>> = OnceLock::new();
static CONTEXT: OnceLock<Mutex<Option<SlackContext>>> = OnceLock::new();

/// Parse + validate then store token+cookie in OS keyring.
/// `token_cookie` format:
///   "<token>:<d_cookie_value>"  OR
///   "<token>:d=<d_cookie>;d-s=<d-s_cookie>"
pub fn register(_lua: &Lua, profile: &str, token_cookie: &str) -> LuaResult<AuthSummary> {
    let (token, _cookie_str) = build_session_token(token_cookie)?;

    let auth = runtime().block_on(async {
        let session = http_client().open_session(&token);
        session.auth_test().await.context("Slack auth.test failed")
    })?;

    // Store the original string so we don't lose cookie formatting
    keyring_entry(profile)?
        .set_password(token_cookie)
        .context("failed to store Slack session secret in keyring")?;

    Ok(AuthSummary::from_auth_test(&auth))
}

/// Load token+cookie from keyring, validate, and set active runtime context.
pub fn init_runtime(_lua: &Lua, profile: String) -> LuaResult<AuthSummary> {
    // If already initialized for same profile, return current auth info.
    {
        let guard = context_cell().lock().unwrap();
        if let Some(ctx) = guard.as_ref() {
            if ctx.profile == profile {
                return Ok(ctx.auth.clone());
            }
        }
    }

    let secret = keyring_entry(&profile)?
        .get_password()
        .context("failed to read Slack session secret from keyring")?;

    let (token, _cookie_str) = build_session_token(&secret)?;

    let auth_test = runtime().block_on(async {
        let session = http_client().open_session(&token);
        session.auth_test().await.context("Slack auth.test failed")
    })?;

    let auth = AuthSummary::from_auth_test(&auth_test);

    // Store active context
    {
        let mut guard = context_cell().lock().unwrap();
        *guard = Some(SlackContext {
            profile: profile.to_string(),
            token,
            auth: auth.clone(),
        });
    }

    Ok(auth)
}

pub(crate) fn active_context() -> Result<SlackContext> {
    let guard = context_cell().lock().unwrap();
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow!("slack.nvim: not initialized; call init_runtime(profile) first"))
}

pub(crate) fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("failed to create tokio runtime"))
}

pub(crate) fn http_client() -> &'static Arc<SlackHyperClient> {
    HTTP_CLIENT.get_or_init(|| {
        let connector =
            SlackClientHyperConnector::new().expect("failed to create Slack hyper connector");
        Arc::new(SlackHyperClient::new(connector))
    })
}

fn context_cell() -> &'static Mutex<Option<SlackContext>> {
    CONTEXT.get_or_init(|| Mutex::new(None))
}

fn keyring_entry(profile: &str) -> Result<Entry> {
    Entry::new(KEYRING_SERVICE, profile).context("failed to create keyring entry")
}

#[derive(Clone, Debug)]
pub(crate) struct SlackContext {
    pub profile: String,
    pub token: SlackApiToken,
    pub auth: AuthSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthSummary {
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub team: Option<String>,
    pub user: Option<String>,
    pub url: Option<String>,
}

impl AuthSummary {
    fn from_auth_test(r: &SlackApiAuthTestResponse) -> Self {
        Self {
            team_id: Some(r.team_id.to_string()),
            user_id: Some(r.user_id.to_string()),
            team: Some(r.team.clone()),
            user: r.user.clone(),
            url: Some(r.url.0.to_string()),
        }
    }
}

/// Builds a SlackApiToken with cookie set (session-token style).
/// SlackApiToken explicitly supports an optional cookie field. :contentReference[oaicite:4]{index=4}
fn build_session_token(token_cookie: &str) -> Result<(SlackApiToken, String)> {
    let (token_str, cookie_str) = token_cookie
        .split_once(':')
        .ok_or_else(|| anyhow!("expected '<token>:<cookie>'"))?;

    let token_value: SlackApiTokenValue = token_str.trim().to_string().into();

    // Accept either:
    // - raw cookie value -> assume it's the `d` cookie
    // - already formatted cookie string "d=...;d-s=..."
    let cookie_trimmed = cookie_str.trim();
    let cookie_header_value = if cookie_trimmed.contains('=') {
        cookie_trimmed.to_string()
    } else {
        format!("d={cookie_trimmed}")
    };

    let cookie_value: SlackApiCookieValue = cookie_header_value.clone().into();
    let token = SlackApiToken::new(token_value).with_cookie(cookie_value);

    Ok((token, cookie_header_value))
}

fn lua_register(lua: &Lua, (profile, token_cookie): (String, String)) -> LuaResult<LuaValue> {
    let auth = register(lua, &profile, &token_cookie)?;
    lua.to_value(&auth).map_err(LuaError::external)
}

fn lua_init_runtime(lua: &Lua, profile: String) -> LuaResult<LuaValue> {
    let auth = init_runtime(lua, profile)?;
    lua.to_value(&auth).map_err(LuaError::external)
}

#[tracing::instrument]
#[mlua::lua_module(skip_memory_check)]
fn slack_client(lua: &Lua) -> LuaResult<mlua::Table> {
    let exports = lua.create_table()?;

    exports.set(
        "init_logging",
        lua.create_function(|_, path: String| {
            logging::setup_logger(&path, "http://localhost:4317")
                .map_err(|e| LuaError::external(format!("{:?}", e)))?;
            Ok(())
        })?,
    )?;

    exports.set("init_runtime", lua.create_function(lua_init_runtime)?)?;
    exports.set("register", lua.create_function(lua_register)?)?;

    Ok(exports)
}
