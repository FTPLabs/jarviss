use once_cell::sync::OnceCell;
  use pv_recorder::{PvRecorder, PvRecorderBuilder};
  use std::sync::atomic::{AtomicBool, Ordering};

  static RECORDER: OnceCell<PvRecorder> = OnceCell::new();
  static IS_RECORDING: AtomicBool = AtomicBool::new(false);

  pub fn init_microphone(device_index: i32, frame_length: u32) -> bool {
      if RECORDER.get().is_some() {
          return true; // already initialized
      }

      let pv_recorder = PvRecorderBuilder::new(frame_length as i32)
          .device_index(device_index)
          .init();

      match pv_recorder {
          Ok(pv) => {
              let _ = RECORDER.set(pv);
              true
          }
          Err(msg) => {
              error!("Failed to initialize pvrecorder.\nError details: {:?}", msg);
              false
          }
      }
  }

  pub fn read_microphone(frame_buffer: &mut [i16]) {
      if RECORDER.get().is_some() {
          let frame = RECORDER.get().unwrap().read();
          match frame {
              Ok(f) => {
                  frame_buffer.copy_from_slice(f.as_slice());
              }
              Err(msg) => {
                  error!("Failed to read audio frame. {:?}", msg);
              }
          }
      }
  }

  pub fn start_recording(device_index: i32, frame_length: u32) -> Result<(), ()> {
      // ensure microphone is initialized — check return value to avoid panic
      if !init_microphone(device_index, frame_length) {
          error!("Cannot start recording: microphone initialization failed");
          return Err(());
      }

      match RECORDER.get().unwrap().start() {
          Ok(_) => {
              info!("START recording from microphone ...");
              IS_RECORDING.store(true, Ordering::SeqCst);
              Ok(())
          }
          Err(msg) => {
              error!("Failed to START audio recording: {}", msg);
              Err(())
          }
      }
  }

  pub fn stop_recording() -> Result<(), ()> {
      if RECORDER.get().is_some() && IS_RECORDING.load(Ordering::SeqCst) {
          match RECORDER.get().unwrap().stop() {
              Ok(_) => {
                  info!("STOP recording from microphone ...");
                  IS_RECORDING.store(false, Ordering::SeqCst);
                  return Ok(());
              }
              Err(msg) => {
                  error!("Failed to STOP audio recording: {}", msg);
                  return Err(());
              }
          }
      }
      Ok(())
  }

  pub fn list_audio_devices() -> Vec<String> {
      match PvRecorderBuilder::default().get_available_devices() {
          Ok(devices) => devices,
          Err(err) => {
              error!("Failed to get audio devices: {}", err);
              Vec::new()
          }
      }
  }

  pub fn get_audio_device_name(idx: i32) -> String {
      if idx == -1 {
          return String::from("System Default");
      }
      let devices = list_audio_devices();
      devices
          .get(idx as usize)
          .cloned()
          .unwrap_or_else(|| devices.first().cloned().unwrap_or_default())
  }
  