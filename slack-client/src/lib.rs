use mlua::prelude::*;
use mlua::Lua;

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
    Ok(exports)
}
