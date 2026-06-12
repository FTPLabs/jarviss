// Core Lua API: log, sleep, print, etc.

  use mlua::{Lua, Table, MultiValue, Value};

  pub fn register(lua: &Lua, jarvis: &Table) -> mlua::Result<()> {

      // _chain: default true — stay in listening mode after command execution
      jarvis.set("_chain", true)?;

      // @ jarvis.log(level, message)
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

      // @ jarvis.print(...)
      // BUG FIX: old code used {:?} debug format (adds quotes around strings).
      // Now maps each Value to its human-readable representation.
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

      // @ jarvis.sleep(ms)
      let sleep_fn = lua.create_function(|_, ms: u64| {
          std::thread::sleep(std::time::Duration::from_millis(ms));
          Ok(())
      })?;
      jarvis.set("sleep", sleep_fn)?;

      // @ jarvis.speak(text)
      // @ jarvis.speak(text) — platform TTS (PowerShell/espeak/say)
      let speak_fn = lua.create_function(|_, text: String| {
          crate::voices::speak_text(&text); // FIX: wired to real TTS
          Ok(())
      })?;
      jarvis.set("speak", speak_fn)?;

      // @ jarvis.set_chain(bool) — control chaining from Lua
      let set_chain_fn = lua.create_function(|lua, value: bool| {
          let globals = lua.globals();
          if let Ok(tbl) = globals.get::<mlua::Table>("jarvis") {
              tbl.set("_chain", value)?;
          }
          Ok(())
      })?;
      jarvis.set("set_chain", set_chain_fn)?;

      Ok(())
  }
  