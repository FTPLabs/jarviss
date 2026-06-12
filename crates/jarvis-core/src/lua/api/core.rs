// Core Lua API: log, sleep, print, speak, chain control

  use mlua::{Lua, Table, MultiValue, Value};

  pub fn register(lua: &Lua, jarvis: &Table) -> mlua::Result<()> {

      // jarvis._chain — controls chaining after execution (default: true = stay listening)
      jarvis.set("_chain", true)?;

      // jarvis.log(level, message)
      let log_fn = lua.create_function(|_, (level, message): (String, String)| {
          match level.to_lowercase().as_str() {
              "debug" => log::debug!("[Lua] {}", message),
              "info"  => log::info!("[Lua] {}", message),
              "warn"  => log::warn!("[Lua] {}", message),
              "error" => log::error!("[Lua] {}", message),
              _       => log::info!("[Lua] {}", message),
          }
          Ok(())
      })?;
      jarvis.set("log", log_fn)?;

      // jarvis.print(...) — human-readable multi-value output (no debug-format quotes)
      let print_fn = lua.create_function(|_, args: MultiValue| {
          let parts: Vec<String> = args.iter()
              .map(|v| match v {
                  Value::Nil        => "nil".to_string(),
                  Value::Boolean(b) => b.to_string(),
                  Value::Integer(i) => i.to_string(),
                  Value::Number(f)  => f.to_string(),
                  Value::String(s)  => s.to_str()
                      .map(|s| s.to_string())
                      .unwrap_or_else(|_| "<invalid utf8>".to_string()),
                  Value::Table(_)    => "<table>".to_string(),
                  Value::Function(_) => "<function>".to_string(),
                  _                  => format!("{:?}", v),
              })
              .collect();
          log::info!("[Lua] {}", parts.join("\t"));
          Ok(())
      })?;
      jarvis.set("print", print_fn)?;

      // jarvis.sleep(ms)
      let sleep_fn = lua.create_function(|_, ms: u64| {
          std::thread::sleep(std::time::Duration::from_millis(ms));
          Ok(())
      })?;
      jarvis.set("sleep", sleep_fn)?;

      // jarvis.speak(text) — platform TTS: PowerShell on Windows, espeak on Linux, say on macOS
      let speak_fn = lua.create_function(|_, text: String| {
          log::info!("[Lua] speak: {}", text);
          crate::voices::speak_text(&text);
          Ok(())
      })?;
      jarvis.set("speak", speak_fn)?;

      // jarvis.set_chain(bool) — set whether to stay in command-listening mode after this script.
      // Default: true (keep listening). Call jarvis.set_chain(false) to return to wake-word mode.
      let set_chain_fn = lua.create_function(|lua, value: bool| {
          let globals = lua.globals();
          if let Ok(jarvis_tbl) = globals.get::<Table>("jarvis") {
              jarvis_tbl.set("_chain", value)?;
          }
          Ok(())
      })?;
      jarvis.set("set_chain", set_chain_fn)?;

      Ok(())
  }
  