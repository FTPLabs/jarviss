use std::fs;
  use std::path::{Path, PathBuf};
  use rand::prelude::*;
  use once_cell::sync::OnceCell;
  use parking_lot::RwLock;

  use crate::{DB, SOUND_DIR, audio, config, time};

  mod structs;
  pub use structs::*;

  static VOICES: OnceCell<Vec<structs::VoiceConfig>> = OnceCell::new();
  static CURRENT_VOICE_ID: OnceCell<RwLock<String>> = OnceCell::new();

  pub fn init(default_voice: &str, language: &str) -> Result<(), String> {
      let voices = scan_voices()?;

      if voices.is_empty() {
          return Err("No voices found".into());
      }

      info!("Loaded {} voice(s): {:?}",
          voices.len(),
          voices.iter().map(|v| &v.voice.id).collect::<Vec<_>>()
      );

      let voice_id = if !default_voice.is_empty() && voices.iter().any(|v| v.voice.id == default_voice) {
          default_voice.to_string()
      } else {
          let auto = voices.iter()
              .find(|v| v.voice.languages.contains(&language.to_string()))
              .or_else(|| voices.first());

          match auto {
              Some(v) => {
                  if default_voice.is_empty() {
                      info!("No voice configured, auto-selected '{}' for language '{}'", v.voice.id, language);
                  } else {
                      warn!("Voice '{}' not found, auto-selected '{}'", default_voice, v.voice.id);
                  }
                  v.voice.id.clone()
              }
              None => return Err("No compatible voice found".into()),
          }
      };

      CURRENT_VOICE_ID.get_or_init(|| RwLock::new(voice_id));
      VOICES.set(voices).map_err(|_| "Voices already initialized")?;

      Ok(())
  }

  pub fn scan_voices() -> Result<Vec<structs::VoiceConfig>, String> {
      let voices_dir = SOUND_DIR.join(config::VOICES_PATH);

      if !voices_dir.exists() {
          return Err(format!("Voices directory not found: {:?}", voices_dir));
      }

      let mut voices = Vec::new();

      let entries = fs::read_dir(&voices_dir)
          .map_err(|e| format!("Failed to read voices directory: {}", e))?;

      for entry in entries.flatten() {
          let voice_path = entry.path();
          if !voice_path.is_dir() {
              continue;
          }

          let toml_path = voice_path.join("voice.toml");
          if !toml_path.exists() {
              warn!("Voice folder {:?} missing voice.toml, skipping", voice_path);
              continue;
          }

          match load_voice_config(&toml_path, &voice_path) {
              Ok(config) => voices.push(config),
              Err(e) => warn!("Failed to load voice {:?}: {}", voice_path, e),
          }
      }

      Ok(voices)
  }

  fn load_voice_config(toml_path: &Path, voice_path: &Path) -> Result<structs::VoiceConfig, String> {
      let content = fs::read_to_string(toml_path)
          .map_err(|e| format!("Failed to read voice.toml: {}", e))?;

      let mut config: structs::VoiceConfig = toml::from_str(&content)
          .map_err(|e| format!("Failed to parse voice.toml: {}", e))?;

      config.path = voice_path.to_path_buf();

      Ok(config)
  }

  pub fn get_current_voice_id() -> String {
      CURRENT_VOICE_ID
          .get()
          .map(|v| v.read().clone())
          .unwrap_or_default()
  }

  pub fn set_voice(id: &str) -> Result<(), String> {
      let voices = VOICES.get().ok_or("Voices not initialized")?;
      if !voices.iter().any(|v| v.voice.id == id) {
          return Err(format!("Voice '{}' not found", id));
      }
      if let Some(current) = CURRENT_VOICE_ID.get() {
          *current.write() = id.to_string();
      }
      Ok(())
  }

  fn get_voice_config() -> Option<&'static structs::VoiceConfig> {
      let voices = VOICES.get()?;
      let current_id = get_current_voice_id();
      voices.iter().find(|v| v.voice.id == current_id)
          .or_else(|| voices.first())
  }

  fn pick_sound(sounds: &[structs::Sound], tag: &str) -> Option<PathBuf> {
      let candidates: Vec<&structs::Sound> = sounds
          .iter()
          .filter(|s| s.tags.contains(&tag.to_string()))
          .collect();

      if candidates.is_empty() {
          return None;
      }

      let picked = candidates.choose(&mut rand::thread_rng())?;
      Some(picked.path.clone())
  }

  fn play_sound_file(path: &PathBuf) {
      if let Err(e) = audio::play_file(path) {
          error!("Failed to play sound {:?}: {}", path, e);
      }
  }

  pub fn play_greet() {
      let Some(voice) = get_voice_config() else { return };
      let time_tag = time::get_time_tag();
      let tag = format!("greet_{}", time_tag);

      // try time-specific greeting first, then generic "greet"
      let path = pick_sound(&voice.sounds, &tag)
          .or_else(|| pick_sound(&voice.sounds, "greet"));

      if let Some(p) = path {
          play_sound_file(&voice.path.join(&p));
      }
  }

  pub fn play_goodbye() {
      let Some(voice) = get_voice_config() else { return };
      if let Some(p) = pick_sound(&voice.sounds, "goodbye") {
          play_sound_file(&voice.path.join(&p));
      }
  }

  pub fn play_reply() {
      let Some(voice) = get_voice_config() else { return };
      if let Some(p) = pick_sound(&voice.sounds, "reply") {
          play_sound_file(&voice.path.join(&p));
      }
  }

  pub fn play_sounds(sounds: &[String], cmd_path: &PathBuf) {
      let mut rng = rand::thread_rng();
      if let Some(sound) = sounds.choose(&mut rng) {
          let sound_path = cmd_path.join(sound);
          if sound_path.exists() {
              play_sound_file(&sound_path);
              return;
          }
          // fallback: try SOUND_DIR
          let sound_path2 = SOUND_DIR.join(sound);
          if sound_path2.exists() {
              play_sound_file(&sound_path2);
          }
      }
  }

  /// Speak text using the platform TTS engine.
  /// Windows: uses built-in System.Speech via PowerShell (no extra deps).
  /// Linux/macOS: uses espeak if available, otherwise logs.
  pub fn speak_text(text: &str) {
      let text = text.replace(''', "''"); // escape single quotes for PowerShell

      #[cfg(target_os = "windows")]
      {
          let script = format!(
              "Add-Type -AssemblyName System.Speech; \
               $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
               $s.Speak('{}');",
              text
          );
          std::thread::spawn(move || {
              let _ = std::process::Command::new("powershell")
                  .args([
                      "-NonInteractive",
                      "-WindowStyle", "Hidden",
                      "-Command", &script,
                  ])
                  .spawn();
          });
      }

      #[cfg(target_os = "linux")]
      {
          let text_owned = text.to_string();
          std::thread::spawn(move || {
              // try espeak-ng first, then espeak
              let ok = std::process::Command::new("espeak-ng")
                  .arg(&text_owned)
                  .spawn()
                  .is_ok();
              if !ok {
                  let _ = std::process::Command::new("espeak")
                      .arg(&text_owned)
                      .spawn();
              }
          });
      }

      #[cfg(target_os = "macos")]
      {
          let text_owned = text.to_string();
          std::thread::spawn(move || {
              let _ = std::process::Command::new("say")
                  .arg(&text_owned)
                  .spawn();
          });
      }
  }

  pub fn get_all_voices() -> Vec<structs::VoiceInfo> {
      VOICES.get()
          .map(|v| v.iter().map(|vc| vc.voice.clone()).collect())
          .unwrap_or_default()
  }
  