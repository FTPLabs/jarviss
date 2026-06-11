use mlua::{Lua, Table, Result as LuaResult};
  use crate::DB;

  /// Register the `jarvis.settings` table into the jarvis global.
  pub fn register(lua: &Lua, jarvis: &Table) -> LuaResult<()> {
      let settings_table = lua.create_table()?;

      // jarvis.settings.get(key) -> value | nil
      let get_fn = lua.create_function(|_, key: String| {
          let value: Option<String> = DB.get().and_then(|db| {
              let s = db.read();
              match key.as_str() {
                  "language"      => Some(s.language.clone()),
                  "voice"         => Some(s.voice.clone()),
                  "microphone"    => Some(s.microphone.to_string()),
                  _               => None,
              }
          });
          Ok(value)
      })?;

      // jarvis.settings.get_language() -> string
      let get_language_fn = lua.create_function(|_, ()| {
          let lang = DB.get()
              .map(|db| db.read().language.clone())
              .unwrap_or_else(|| "en".to_string());
          Ok(lang)
      })?;

      // jarvis.settings.get_voice() -> string
      let get_voice_fn = lua.create_function(|_, ()| {
          let voice = DB.get()
              .map(|db| db.read().voice.clone())
              .unwrap_or_default();
          Ok(voice)
      })?;

      settings_table.set("get", get_fn)?;
      settings_table.set("get_language", get_language_fn)?;
      settings_table.set("get_voice", get_voice_fn)?;

      jarvis.set("settings", settings_table)?;
      Ok(())
  }
  