use mlua::prelude::*;

mod ecs;
mod slack;
mod storage;

cfg_if::cfg_if! {
    if #[cfg(feature = "telemetry")] {
        use slack_telemetry as logging;
    } else {
        mod log;
        use log as logging;
    }
}

fn anyhow_to_lua(err: anyhow::Error) -> LuaError {
    // {:#} keeps the anyhow context chain which is very useful in Lua
    LuaError::external(format!("{:#}", err))
}

fn lua_register(lua: &Lua, (profile, token): (String, String)) -> LuaResult<LuaValue> {
    let auth = ecs::world::with_world(|world| {
        ecs::systems::auth::register_profile(world, &profile, &token)
    })
    .map_err(anyhow_to_lua)?;

    lua.to_value(&auth).map_err(LuaError::external)
}

fn lua_init_runtime(lua: &Lua, profile: String) -> LuaResult<LuaValue> {
    let auth = ecs::world::with_world(|world| ecs::systems::auth::init_profile(world, &profile))
        .map_err(anyhow_to_lua)?;

    lua.to_value(&auth).map_err(LuaError::external)
}

fn lua_conversations(lua: &Lua, limit: Option<u16>) -> LuaResult<LuaValue> {
    let convos = ecs::world::with_world(|world| {
        ecs::systems::conversations::list_conversations(world, limit)
    })
    .map_err(anyhow_to_lua)?;

    lua.to_value(&convos).map_err(LuaError::external)
}

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

    // NOTE: register(profile, token) - token only (no cookie)
    exports.set("register", lua.create_function(lua_register)?)?;
    exports.set("init_runtime", lua.create_function(lua_init_runtime)?)?;
    exports.set("conversations", lua.create_function(lua_conversations)?)?;

    Ok(exports)
}
