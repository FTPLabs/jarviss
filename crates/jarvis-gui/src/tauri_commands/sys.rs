use sysinfo::{System, Pid, ProcessRefreshKind, RefreshKind, CpuRefreshKind, Components};
  use peak_alloc::PeakAlloc;
  use std::sync::Mutex;
  use once_cell::sync::Lazy;
  use std::path::PathBuf;
  use tauri::Manager; // FIX: required for AppHandle::path() in Tauri v2

  #[global_allocator]
  static PEAK_ALLOC: PeakAlloc = PeakAlloc;

  static SYS: Lazy<Mutex<System>> = Lazy::new(|| {
      Mutex::new(System::new_with_specifics(
          RefreshKind::nothing()
              .with_processes(ProcessRefreshKind::nothing().with_memory().with_cpu())
              .with_cpu(CpuRefreshKind::everything())
      ))
  });

  static COMPONENTS: Lazy<Mutex<Components>> = Lazy::new(|| {
      Mutex::new(Components::new_with_refreshed_list())
  });

  const JARVIS_APP_NAME: &str = "jarvis-app";

  /// Find jarvis-app process and return its PID
  fn find_jarvis_app_pid(sys: &System) -> Option<Pid> {
      for (pid, process) in sys.processes() {
          let name = process.name().to_string_lossy().to_lowercase();
          if name.contains(JARVIS_APP_NAME) {
              return Some(*pid);
          }
      }
      None
  }

  /// Resolve the path to jarvis-app.exe by trying multiple candidate locations:
  /// 1. JARVIS_APP_PATH env-var override (for dev/testing)
  /// 2. Same directory as jarvis-gui.exe (portable / NSIS install)
  /// 3. Tauri resource_dir (for bundled resources in some install layouts)
  fn resolve_jarvis_app_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
      #[cfg(target_os = "windows")]
      let exe_name = "jarvis-app.exe";
      #[cfg(not(target_os = "windows"))]
      let exe_name = "jarvis-app";

      // 1. Env-var override
      if let Ok(override_path) = std::env::var("JARVIS_APP_PATH") {
          let p = PathBuf::from(override_path);
          if p.exists() {
              info!("Using JARVIS_APP_PATH override: {}", p.display());
              return Ok(p);
          }
          warn!("JARVIS_APP_PATH is set but file not found: {}", p.display());
      }

      // 2. Same directory as current exe (works for NSIS install & portable)
      if let Ok(exe_path) = std::env::current_exe() {
          if let Some(exe_dir) = exe_path.parent() {
              let candidate = exe_dir.join(exe_name);
              if candidate.exists() {
                  info!("Found jarvis-app next to GUI exe: {}", candidate.display());
                  return Ok(candidate);
              }
              debug!("Not found at exe_dir: {}", candidate.display());
          }
      }

      // 3. Tauri resource directory (e.g. on macOS or certain Windows layouts)
      if let Ok(resource_dir) = app_handle.path().resource_dir() {
          let candidate = resource_dir.join(exe_name);
          if candidate.exists() {
              info!("Found jarvis-app in resource_dir: {}", candidate.display());
              return Ok(candidate);
          }
          debug!("Not found in resource_dir: {}", candidate.display());
      }

      Err(format!(
          "jarvis-app not found. Expected it next to jarvis-gui.exe or in the resource directory. \
  Set JARVIS_APP_PATH env var to override."
      ))
  }

  #[derive(serde::Serialize)]
  pub struct JarvisAppStats {
      pub running: bool,
      pub ram_mb: u64,
      pub cpu_usage: f32,
  }

  #[tauri::command]
  pub fn get_jarvis_app_stats() -> JarvisAppStats {
      let mut sys = SYS.lock().unwrap();
      sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
      
      if let Some(pid) = find_jarvis_app_pid(&sys) {
          if let Some(proc) = sys.process(pid) {
              return JarvisAppStats {
                  running: true,
                  ram_mb: proc.memory() / 1024 / 1024,
                  cpu_usage: proc.cpu_usage(),
              };
          }
      }
      
      JarvisAppStats { running: false, ram_mb: 0, cpu_usage: 0.0 }
  }

  #[tauri::command]
  pub fn get_current_ram_usage() -> u64 {
      let mut sys = SYS.lock().unwrap();
      sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
      
      if let Some(pid) = find_jarvis_app_pid(&sys) {
          if let Some(proc) = sys.process(pid) {
              return proc.memory() / 1024 / 1024;
          }
      }
      0
  }

  #[tauri::command]
  pub fn is_jarvis_app_running() -> bool {
      let mut sys = SYS.lock().unwrap();
      sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
      find_jarvis_app_pid(&sys).is_some()
  }

  #[tauri::command]
  pub fn get_cpu_temp() -> String {
      let mut components = COMPONENTS.lock().unwrap();
      components.refresh(true);
      
      for component in components.iter() {
          let label = component.label().to_lowercase();
          if label.contains("cpu") || label.contains("core") || label.contains("package") {
              if let Some(temp) = component.temperature() {
                  return format!("{:.1}", temp);
              }
          }
      }
      
      if let Some(component) = components.iter().next() {
          if let Some(temp) = component.temperature() {
              return format!("{:.1}", temp);
          }
      }
      
      String::from("N/A")
  }

  #[tauri::command]
  pub fn get_cpu_usage() -> f32 {
      let mut sys = SYS.lock().unwrap();
      sys.refresh_cpu_all();
      std::thread::sleep(std::time::Duration::from_millis(200));
      sys.refresh_cpu_all();
      sys.global_cpu_usage()
  }

  #[tauri::command]
  pub fn get_peak_ram_usage() -> String {
      format!("{}", PEAK_ALLOC.peak_usage_as_gb())
  }

  #[tauri::command]
  pub fn run_jarvis_app(app_handle: tauri::AppHandle) -> Result<(), String> {
      let jarvis_app_path = resolve_jarvis_app_path(&app_handle)?;
      
      info!("Launching jarvis-app: {}", jarvis_app_path.display());
      
      std::process::Command::new(&jarvis_app_path)
          .spawn()
          .map_err(|e| format!("Failed to start jarvis-app: {}", e))?;
      
      Ok(())
  }
  