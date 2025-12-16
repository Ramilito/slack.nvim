use mlua::prelude::*;
use mlua::Lua;

// #[tracing::instrument]
#[mlua::lua_module(skip_memory_check)]
fn slack_client(lua: &Lua) -> LuaResult<mlua::Table> {
    let exports = lua.create_table()?;

    Ok(exports)
}
