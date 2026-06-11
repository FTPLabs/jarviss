// Core Lua API: log, sleep, print, etc.

  use mlua::{Lua, Table, MultiValue, Value};

  pub fn register(lua: &Lua, jarvis: &Table) -> mlua::Result<()> {

      // @ jarvis.log(level, message)
      // log something
      let log_fn = lua.create_function(|_, (level, message): (String, String)| {
          match level.to_lowercase().as_str() {
              "debug" => log::debug!("[Lua] {}", message),
              "info" => log::info!("[Lua] {}", message),
              "warn" => log::warn!("[Lua] {}", message),
              "error" => log::error!("[Lua] {}", message),
              _ => log::info!("[Lua] {}", message),
          }
          Ok(())
      })?;
      jarvis.set("log", log_fn)?;
      
      // @ jarvis.print(...)
      // simple print — BUG FIX: was using {:?} debug format which adds quotes
      // around strings. Now uses display format for primitives.
      let print_fn = lua.create_function(|_, args: MultiValue| {
          let parts: Vec<String> = args.iter()
              .map(|v| lua_value_to_string(v))
              .collect();
          log::info!("[Lua] {}", parts.join("\t"));
          Ok(())
      })?;
      jarvis.set("print", print_fn)?;
      
      // @ jarvis.sleep(ms)
      // ..zZZ
      let sleep_fn = lua.create_function(|_, ms: u64| {
          std::thread::sleep(std::time::Duration::from_millis(ms));
          Ok(())
      })?;
      jarvis.set("sleep", sleep_fn)?;
      
      // @ jarvis.speak(text)
      // @TODO: update when TTS will be implemented
      let speak_fn = lua.create_function(|_, text: String| {
          log::info!("[Lua] SPEAK (TTS not yet implemented): {}", text);
          // pass
          Ok(())
      })?;
      jarvis.set("speak", speak_fn)?;
      
      Ok(())
  }

  /// Convert a Lua Value to a human-readable string (display format, not debug)
  fn lua_value_to_string(v: &Value) -> String {
      match v {
          Value::Nil => "nil".to_string(),
          Value::Boolean(b) => b.to_string(),
          Value::Integer(i) => i.to_string(),
          Value::Number(f) => f.to_string(),
          Value::String(s) => s.to_str().unwrap_or("<invalid utf8>").to_string(),
          Value::Table(_) => "<table>".to_string(),
          Value::Function(_) => "<function>".to_string(),
          _ => format!("{:?}", v),
      }
  }
  