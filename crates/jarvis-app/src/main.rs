use std::sync::Arc;
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::sync::mpsc;

  use jarvis_core::{
      audio, audio_processing, commands, config, db, listener, recorder, stt, intent,
      ipc::{self, IpcAction},
      i18n, voices, models, slots,
      APP_CONFIG_DIR, APP_LOG_DIR, COMMANDS_LIST, DB,
  };

  #[macro_use]
  extern crate simple_log;
  mod log;

  mod app;

  #[cfg(not(target_os = "macos"))]
  mod tray;

  static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
  static SHOULD_RELOAD_COMMANDS: AtomicBool = AtomicBool::new(false);
  static IS_MUTED: AtomicBool = AtomicBool::new(false);

  fn main() -> Result<(), String> {
      config::init_dirs()?;
      log::init_logging()?;

      info!("Starting Jarvis v{} ...", config::APP_VERSION.unwrap_or("unknown"));
      info!("Config directory is: {}", APP_CONFIG_DIR.get().unwrap().display());
      info!("Log directory is: {}", APP_LOG_DIR.get().unwrap().display());

      let settings = db::init();

      DB.set(settings.arc().clone())
          .expect("DB already initialized");

      let voice_id = settings.lock().voice.clone();
      let language = settings.lock().language.clone();
      if let Err(e) = voices::init(&voice_id, &language) {
          warn!("Failed to init voices: {}", e);
      }

      i18n::init(&settings.lock().language);

      if recorder::init().is_err() {
          app::close(1);
      }

      if let Err(e) = models::init() {
          warn!("Models registry init failed: {}", e);
      }

      if stt::init().is_err() {
          app::close(1);
      }

      info!("Initializing commands.");
      load_commands();

      if audio::init().is_err() {
          app::close(1);
      }

      if let Err(e) = listener::init() {
          error!("Wake-word engine init failed: {}", e);
          app::close(1);
      }

      let rt = Arc::new(
          tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
      );

      rt.block_on(async {
          let cmds = COMMANDS_LIST.read();
          if let Err(e) = intent::init(&cmds).await {
              error!("Failed to initialize intent classifier: {}", e);
              app::close(1);
          }
      });

      slots::init().map_err(|e| error!("Slot extraction init failed: {}", e)).ok();

      info!("Initializing audio processing...");
      if let Err(e) = audio_processing::init() {
          warn!("Audio processing init failed: {}", e);
      }

      info!("Initializing IPC...");
      ipc::init();

      let (text_cmd_tx, text_cmd_rx) = mpsc::channel::<String>();

      ipc::set_action_handler(move |action| {
          match action {
              IpcAction::Stop => {
                  info!("Received stop command from GUI");
                  SHOULD_STOP.store(true, Ordering::SeqCst);
              }
              IpcAction::ReloadCommands => {
                  info!("Received reload commands request — scheduling hot-reload");
                  SHOULD_RELOAD_COMMANDS.store(true, Ordering::SeqCst);
              }
              IpcAction::SetMuted { muted } => {
                  info!("Mute state changed: {}", muted);
                  IS_MUTED.store(muted, Ordering::SeqCst);
              }
              IpcAction::TextCommand { text } => {
                  info!("Received text command: {}", text);
                  if let Err(e) = text_cmd_tx.send(text) {
                      error!("Failed to send text command to app: {}", e);
                  }
              }
              IpcAction::Ping => {
                  // handled internally by IPC server
              }
          }
      });

      let ipc_rt = Arc::clone(&rt);
      std::thread::spawn(move || {
          ipc_rt.block_on(ipc::start_server());
      });

      let app_rt = Arc::clone(&rt);
      std::thread::spawn(move || {
          let _ = app::start(text_cmd_rx, &app_rt);
      });

      tray::init_blocking(settings);

      Ok(())
  }

  /// Parse commands from disk and store in the global COMMANDS_LIST.
  fn load_commands() {
      match commands::parse_commands() {
          Ok(cmds) => {
              info!("Commands loaded. Count: {}, paths: {:?}", cmds.len(), commands::list_paths(&cmds));
              let mut list = COMMANDS_LIST.write();
              *list = cmds;
          }
          Err(e) => {
              warn!("Failed to parse commands: {}. Starting with empty command list.", e);
          }
      }
  }

  pub fn should_stop() -> bool {
      SHOULD_STOP.load(Ordering::SeqCst)
  }

  pub fn should_reload_commands() -> bool {
      SHOULD_RELOAD_COMMANDS.swap(false, Ordering::SeqCst)
  }

  pub fn is_muted() -> bool {
      IS_MUTED.load(Ordering::SeqCst)
  }
  