use bevy::prelude::*;
use mlua::prelude::*;

use crc32fast::Hasher;
use std::{
    cell::RefCell,
    fs::read_to_string,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::runtime::env_define;

const LOG_SCRIPT_TARGET: &str = "lua";

#[derive(Default, Clone)]
pub(crate) struct CommonAppData {
    /// Incrementally saves hashes of loaded scripts.
    pub(crate) hasher: Rc<RefCell<Hasher>>,
}

pub(crate) fn create_environment(lua: &Lua) -> LuaResult<()> {
    let table = lua.globals();
    table.clear()?;

    table.set("define", lua.create_function(env_define)?)?;
    table.set("require", lua.create_function(env_require)?)?;
    table.set("info", lua.create_function(env_info)?)?;
    table.set("warn", lua.create_function(env_warn)?)?;
    table.set("error", lua.create_function(env_error)?)?;

    Ok(())
}

fn env_require(lua: &Lua, filename: String) -> LuaResult<LuaValue> {
    let filename = filename.replace('.', "/");

    // TODO: remove any ../ or backpedalling and only allow local paths

    let path = Path::new(&filename);
    let full_path: PathBuf = Path::new("assets").join(path).with_extension("lua");
    let full_path = full_path.canonicalize()?;

    let script_content = read_to_string(full_path)?;
    let app_data = lua
        .app_data_ref::<CommonAppData>()
        .expect("AppData not present!");

    app_data
        .hasher
        .borrow_mut()
        .update(script_content.as_bytes());

    let chunk = lua.load(&script_content);
    let result = chunk.eval::<mlua::Value>()?;

    Ok(result)
}

fn env_info(_: &Lua, input: String) -> LuaResult<()> {
    info!(target:LOG_SCRIPT_TARGET, input);
    Ok(())
}

fn env_warn(_: &Lua, input: String) -> LuaResult<()> {
    warn!(target:LOG_SCRIPT_TARGET, input);
    Ok(())
}

fn env_error(_: &Lua, input: String) -> LuaResult<()> {
    error!(target:LOG_SCRIPT_TARGET, input);
    Ok(())
}
