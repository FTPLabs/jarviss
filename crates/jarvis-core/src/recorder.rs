mod pvrecorder;

  use once_cell::sync::OnceCell;

  use crate::{config, config::structs::RecorderType, DB};

  static RECORDER_TYPE: OnceCell<RecorderType> = OnceCell::new();
  static FRAME_LENGTH: OnceCell<u32> = OnceCell::new();

  pub fn init() -> Result<(), ()> {
      // @TODO: expose recorder_type as a user setting in the DB
      RECORDER_TYPE.set(config::DEFAULT_RECORDER_TYPE).unwrap();

      info!("Loading recorder ...");
      info!("Available audio devices:\n{:?}", get_audio_devices());

      match RECORDER_TYPE.get().unwrap() {
          RecorderType::PvRecorder => {
              info!("Initializing PvRecorder recording backend.");
              FRAME_LENGTH.set(512u32).unwrap();
              let selected_microphone = get_selected_microphone_index();
              match pvrecorder::init_microphone(
                  selected_microphone,
                  *FRAME_LENGTH.get().unwrap(),
              ) {
                  false => {
                      error!("PvRecorder initialization failed.");
                      return Err(());
                  }
                  _ => {
                      info!(
                          "Recorder initialization success. Listening to microphone ({}): {}",
                          selected_microphone,
                          get_audio_device_name(selected_microphone)
                      );
                  }
              }
          }
          RecorderType::PortAudio => {
              error!("PortAudio backend is not yet implemented. Please use PvRecorder.");
              return Err(());
          }
          RecorderType::Cpal => {
              error!("CPAL backend is not yet implemented. Please use PvRecorder.");
              return Err(());
          }
      }

      Ok(())
  }

  pub fn read_microphone(frame_buffer: &mut [i16]) {
      match RECORDER_TYPE.get().unwrap() {
          RecorderType::PvRecorder => {
              pvrecorder::read_microphone(frame_buffer);
          }
          RecorderType::PortAudio => {
              error!("PortAudio backend is not implemented; cannot read microphone.");
          }
          RecorderType::Cpal => {
              error!("CPAL backend is not implemented; cannot read microphone.");
          }
      }
  }

  pub fn start_recording() -> Result<(), ()> {
      match RECORDER_TYPE.get().unwrap() {
          RecorderType::PvRecorder => pvrecorder::start_recording(),
          RecorderType::PortAudio | RecorderType::Cpal => {
              error!("start_recording: backend not implemented.");
              Err(())
          }
      }
  }

  pub fn stop_recording() -> Result<(), ()> {
      match RECORDER_TYPE.get().unwrap() {
          RecorderType::PvRecorder => pvrecorder::stop_recording(),
          RecorderType::PortAudio | RecorderType::Cpal => Ok(()),
      }
  }

  pub fn get_selected_microphone_index() -> i32 {
      DB.get()
          .map(|db| db.read().microphone)
          .unwrap_or(-1)
  }

  pub fn get_audio_devices() -> Vec<String> {
      pvrecorder::get_audio_devices()
  }

  pub fn get_audio_device_name(index: i32) -> String {
      pvrecorder::get_audio_device_name(index)
  }
  