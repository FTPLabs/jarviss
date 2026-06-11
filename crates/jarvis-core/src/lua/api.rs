use mlua::Lua;

use crate::lua::{CommandContext};
use super::sandbox::SandboxLevel;

pub mod core;
pub mod audio;
pub mod context;
pub mod http;
pub mod fs;
pub mod state;
pub mod system;
pub mod settings;

/// Register all jarvis.* Lua APIs into the given Lua state.
/// Called once per script execution, after sandbox setup.
pub fn register(lua: &Lua, ctx: CommandContext, sandbox: SandboxLevel) -> mlua::Result<()> {
    // create top-level `jarvis` table
    let jarvis = lua.create_table()?;

    core::register(lua, &jarvis)?;
    audio::register(lua, &jarvis)?;
    context::register(lua, &jarvis, &ctx)?;
    http::register(lua, &jarvis)?;
    fs::register(lua, &jarvis, &ctx.command_path, sandbox)?;
    state::register(lua, &jarvis, &ctx.command_path)?;
    system::register(lua, &jarvis, sandbox)?;
    settings::register(lua, &jarvis)?;

    // expose as a global
    lua.globals().set("jarvis", jarvis)?;
    Ok(())
}
