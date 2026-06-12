use mlua::{Lua, Value, StdLib};
  use std::path::PathBuf;
  use std::time::Duration;
  use std::fs;

  use super::sandbox::SandboxLevel;
  use super::error::LuaError;
  use super::{CommandContext, CommandResult};
  use super::api;

  /// Marker used by the timeout hook — must be unique enough to not
  /// collide with user script errors.
  const TIMEOUT_MARKER: &str = "__jarvis_script_timeout__";

  pub struct LuaEngine {
      lua: Lua,
      sandbox: SandboxLevel,
  }

  impl LuaEngine {
      pub fn new(sandbox: SandboxLevel) -> Result<Self, LuaError> {
          // select which standard libraries to load based on sandbox access level
          let std_libs = match sandbox {
              SandboxLevel::Minimal => {
                  StdLib::TABLE | StdLib::STRING | StdLib::MATH
              }
              SandboxLevel::Standard => {
                  StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8
              }
              SandboxLevel::Full => {
                  StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8 | StdLib::OS
              }
          };

          let lua = Lua::new_with(std_libs, mlua::LuaOptions::default())
              .map_err(|e| LuaError::InitError(e.to_string()))?;

          // remove dangerous globals regardless of sandbox level
          {
              let globals = lua.globals();

              // always remove these — prevent arbitrary code loading
              let _ = globals.set("loadfile", Value::Nil);
              let _ = globals.set("dofile", Value::Nil);
              let _ = globals.set("load", Value::Nil);
              let _ = globals.set("loadstring", Value::Nil);

              // SECURITY: block require/package to prevent loading native C libs
              // (e.g. require('evil.dll') could escape the sandbox entirely)
              let _ = globals.set("require", Value::Nil);
              let _ = globals.set("package", Value::Nil);

              // remove io unless full sandbox
              if !matches!(sandbox, SandboxLevel::Full) {
                  let _ = globals.set("io", Value::Nil);
              }

              // remove dangerous os.* even in full mode
              if matches!(sandbox, SandboxLevel::Full) {
                  if let Ok(os) = globals.get::<mlua::Table>("os") {
                      let _ = os.set("execute", Value::Nil);
                      let _ = os.set("exit", Value::Nil);
                      let _ = os.set("remove", Value::Nil);
                      let _ = os.set("rename", Value::Nil);
                      let _ = os.set("setlocale", Value::Nil);
                  }
              }
          }

          Ok(Self { lua, sandbox })
      }

      pub fn execute(
          &self,
          script_path: &PathBuf,
          context: CommandContext,
          timeout: Duration,
      ) -> Result<CommandResult, LuaError> {
          let script = fs::read_to_string(script_path)
              .map_err(LuaError::IoError)?;

          // register jarvis API
          api::register(&self.lua, context, self.sandbox)?;

          // install timeout hook
          let start = std::time::Instant::now();
          let timeout_marker = TIMEOUT_MARKER.to_string();
          self.lua.set_hook(mlua::HookTriggers { every_nth_instruction: Some(1000), ..Default::default() }, move |_, _| {
              if start.elapsed() >= timeout {
                  Err(mlua::Error::runtime(timeout_marker.clone()))
              } else {
                  Ok(mlua::VmState::Continue)
              }
          });

          let result = self.lua.load(&script).eval::<mlua::MultiValue>();

          // remove hook after execution
          self.lua.remove_hook();

          match result {
              Ok(_) => {
                    // FIX: read jarvis._chain from Lua globals after execution
                    let chain = self.lua.globals()
                        .get::<mlua::Table>("jarvis")
                        .and_then(|t| t.get::<bool>("_chain"))
                        .unwrap_or(true);
                    Ok(CommandResult { chain, ..Default::default() })
                }
              Err(mlua::Error::RuntimeError(msg)) if msg.contains(TIMEOUT_MARKER) => {
                  Err(LuaError::Timeout)
              }
              Err(e) => Err(LuaError::RuntimeError(e.to_string())),
          }
      }
  }

  /// Convenience function: create engine, execute script, return result.
  pub fn execute(
      script_path: &PathBuf,
      context: CommandContext,
      sandbox: SandboxLevel,
      timeout: Duration,
  ) -> Result<CommandResult, LuaError> {
      let engine = LuaEngine::new(sandbox)?;
      engine.execute(script_path, context, timeout)
  }
  