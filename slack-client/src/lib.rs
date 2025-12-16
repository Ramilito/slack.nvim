use anyhow::Context;
use mlua::prelude::*;
use mlua::Lua;

use crate::client::test_auth;
use crate::client::AuthInfo;
use crate::store::load_token;
use crate::store::save_token;
use crate::token::build_slack_token;
use crate::token::parse_session_token;

mod client;
mod store;
mod token;

cfg_if::cfg_if! {
    if #[cfg(feature = "telemetry")] {
        use slack_telemetry as logging;
    } else {
        mod log;
        use log as logging;
    }
}

#[tracing::instrument]
fn init_runtime(_lua: &Lua, context_name: Option<String>) -> LuaResult<(bool, String)> {
    Ok((true, String::new()))
}

fn register_session_token(lua: &Lua, args: (String, Option<String>)) -> LuaResult<LuaTable> {
    let (token_string, profile_opt) = args;
    let profile = profile_opt.unwrap_or_else(|| "default".to_string());

    // 1. Parse wee-slack style token string
    let parts = parse_session_token(&token_string);
    let slack_token = build_slack_token(&parts);

    // 2. Call auth.test to verify token + find team/user
    let auth_info = test_auth(slack_token)
        .with_context(|| "Slack auth.test failed for provided token") // anyhow
        .map_err(LuaError::external)?;

    // 3. Store the original string in OS keyring
    save_token(&profile, &token_string)
        .with_context(|| format!("Failed to store token for profile '{profile}'"))
        .map_err(LuaError::external)?;

    // 4. Build Lua return table
    auth_info_to_table(lua, Some(profile), auth_info, None)
}

fn test_connection(lua: &Lua, profile_opt: Option<String>) -> LuaResult<LuaTable> {
    let profile = profile_opt.unwrap_or_else(|| "default".to_string());

    // 1. Load token from keyring
    let token_string = match load_token(&profile) {
        Ok(token) => token,
        Err(err) => {
            // Return a structured error table instead of raising
            let t = lua.create_table()?;
            t.set("ok", false)?;
            t.set("profile", profile)?;
            t.set("error", format!("failed_to_load_token: {err}"))?;
            return Ok(t);
        }
    };

    // 2. Parse + build Slack token
    let parts = parse_session_token(&token_string);
    let slack_token = build_slack_token(&parts);

    // 3. Run auth.test
    match test_auth(slack_token) {
        Ok(auth_info) => auth_info_to_table(lua, Some(profile), auth_info, None),
        Err(err) => auth_info_to_table(
            lua,
            Some(profile),
            // we don't have auth info here, so fake minimal
            AuthInfo {
                user_id: String::new(),
                team_id: String::new(),
                user: None,
                team: String::new(),
                bot_id: None,
                url: String::new(),
            },
            Some(err.to_string()),
        ),
    }
}

fn auth_info_to_table(
    lua: &Lua,
    profile: Option<String>,
    auth: AuthInfo,
    error: Option<String>,
) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;

    t.set("ok", error.is_none())?;
    if let Some(profile) = profile {
        t.set("profile", profile)?;
    }

    if let Some(err) = error {
        t.set("error", err)?;
    }

    t.set("user_id", auth.user_id)?;
    t.set("team_id", auth.team_id)?;
    t.set("user", auth.user)?;
    t.set("team", auth.team)?;
    t.set("bot_id", auth.bot_id)?;
    t.set("url", auth.url)?;

    Ok(t)
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

    exports.set("init_runtime", lua.create_function(init_runtime)?)?;
    exports.set(
        "register_session_token",
        lua.create_function(register_session_token)?,
    )?;
    // exports.set(
    //     "register_session_token",
    //     lua.create_function(
    //         |lua, (token_string, profile_opt): (String, Option<String>)| {
    //             register_session_token(lua, token_string, profile_opt)
    //         },
    //     )?,
    // )?;

    exports.set(
        "test_connection",
        lua.create_function(|lua, profile_opt: Option<String>| test_connection(lua, profile_opt))?,
    )?;

    Ok(exports)
}
