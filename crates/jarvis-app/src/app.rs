use std::sync::mpsc::Receiver;
  use std::time::SystemTime;

  use jarvis_core::{audio_buffer::AudioRingBuffer, audio_processing, commands, config, listener, recorder, stt, COMMANDS_LIST, intent, voices, ipc::{self, IpcEvent}, i18n, slots};
  use rand::seq::SliceRandom;

  use crate::{should_stop, is_muted, should_reload_commands};

  // VAD state machine
  #[derive(Debug, Clone, Copy, PartialEq)]
  enum VadState {
      WaitingForVoice,
      VoiceActive,
  }

  pub fn start(text_cmd_rx: Receiver<String>, rt: &tokio::runtime::Runtime) -> Result<(), ()> {
      main_loop(text_cmd_rx, rt)
  }

  fn main_loop(text_cmd_rx: Receiver<String>, rt: &tokio::runtime::Runtime) -> Result<(), ()> {
      let frame_length: usize = 512;
      let sample_rate: usize = 16000;
      let mut frame_buffer: Vec<i16> = vec![0; frame_length];

      // ring buffer: keeps last 5 seconds of audio (pre-roll)
      let mut audio_buffer = AudioRingBuffer::new(5.0, frame_length, sample_rate);

      let mut vad_state = VadState::WaitingForVoice;
      let mut silence_frames: u32 = 0;

      // 1.5 seconds of silence = end of speech (pre-wake-word)
      let silence_threshold: u32 = ((1.5 * sample_rate as f32) / frame_length as f32) as u32;

      voices::play_greet();

      match recorder::start_recording() {
          Ok(_) => info!("Recording started. Microphone: {}",
              recorder::get_audio_device_name(recorder::get_selected_microphone_index())),
          Err(_) => {
              error!("Cannot start recording.");
              return Err(());
          }
      }

      ipc::send(IpcEvent::Idle);

      // ### WAKE WORD DETECTION LOOP
      'wake_word: loop {
          if should_stop() {
              info!("Stop signal received, shutting down...");
              voices::play_goodbye();
              ipc::send(IpcEvent::Stopping);
              break;
          }

          // Hot-reload commands if requested
          if should_reload_commands() {
              info!("Hot-reloading commands from disk...");
              match commands::parse_commands() {
                  Ok(cmds) => {
                      info!("Commands reloaded. Count: {}", cmds.len());
                      let mut list = COMMANDS_LIST.write();
                      *list = cmds;
                  }
                  Err(e) => warn!("Command reload failed: {}", e),
              }
          }

          if let Ok(text) = text_cmd_rx.try_recv() {
              // Skip text commands when muted
              if !is_muted() {
                  process_text_command(&text, rt);
              }
              continue 'wake_word;
          }

          recorder::read_microphone(&mut frame_buffer);

          // Skip all audio processing and wake-word detection when muted
          if is_muted() {
              continue 'wake_word;
          }

          let processed = audio_processing::process(&frame_buffer);

          match vad_state {
              VadState::WaitingForVoice => {
                  audio_buffer.push(&frame_buffer);

                  if processed.is_voice {
                      info!("VAD: Voice started, flushing {} buffered frames", audio_buffer.len());
                      for buffered_frame in audio_buffer.drain_all() {
                          listener::data_callback(&buffered_frame);
                      }
                      vad_state = VadState::VoiceActive;
                      silence_frames = 0;
                  }
              }

              VadState::VoiceActive => {
                  // dual-feed STT in parallel with wake word detector
                  let _ = stt::recognize(&frame_buffer, false);

                  if let Some(_keyword_index) = listener::data_callback(&frame_buffer) {
                      info!("Wake word activated!");
                      ipc::send(IpcEvent::WakeWordDetected);

                      stt::reset_wake_recognizer();
                      audio_processing::reset();

                      // brief sniff: keep feeding STT while transitioning
                      let sniff_frames = ((0.3 * sample_rate as f32) / frame_length as f32) as u32;
                      for _ in 0..sniff_frames {
                          recorder::read_microphone(&mut frame_buffer);
                          audio_processing::process(&frame_buffer);
                          stt::recognize(&frame_buffer, false);
                      }

                      ipc::send(IpcEvent::Listening);
                      recognize_command(&mut frame_buffer, rt, frame_length, sample_rate, true);

                      vad_state = VadState::WaitingForVoice;
                      silence_frames = 0;
                      audio_buffer.clear();
                      stt::reset_wake_recognizer();
                      stt::reset_speech_recognizer();
                      audio_processing::reset();
                      ipc::send(IpcEvent::Idle);

                      continue 'wake_word;
                  }

                  if processed.is_voice {
                      silence_frames = 0;
                  } else {
                      silence_frames += 1;
                      if silence_frames > silence_threshold {
                          debug!("VAD: Silence timeout, returning to wait state");
                          vad_state = VadState::WaitingForVoice;
                          silence_frames = 0;
                          stt::reset_wake_recognizer();
                          stt::reset_speech_recognizer();
                      }
                  }
              }
          }
      }

      recorder::stop_recording().ok();
      // NOTE: IpcEvent::Stopping was already sent inside the loop before break

      Ok(())
  }


  fn recognize_command(
      frame_buffer: &mut [i16],
      rt: &tokio::runtime::Runtime,
      frame_length: usize,
      sample_rate: usize,
      prefed_audio: bool,
  ) {
      let mut audio_buffer = AudioRingBuffer::new(2.0, frame_length, sample_rate);
      let mut vad_state = if prefed_audio {
          VadState::VoiceActive
      } else {
          VadState::WaitingForVoice
      };
      let mut silence_frames: u32 = 0;
      let mut start = SystemTime::now();
      let mut first_recognition = prefed_audio;

      // 5 seconds silence to exit command mode (user might pause to think)
      let silence_threshold: u32 = ((5.0 * sample_rate as f32) / frame_length as f32) as u32;

      loop {
          if crate::should_stop() {
              return;
          }

          recorder::read_microphone(frame_buffer);
          let processed = audio_processing::process(frame_buffer);

          match vad_state {
              VadState::WaitingForVoice => {
                  audio_buffer.push(frame_buffer);

                  if processed.is_voice {
                      for buffered_frame in audio_buffer.drain_all() {
                          stt::recognize(&buffered_frame, false);
                      }
                      vad_state = VadState::VoiceActive;
                      silence_frames = 0;
                  } else {
                      silence_frames += 1;
                      if silence_frames > silence_threshold {
                          info!("Long silence detected, returning to wake word mode.");
                          return;
                      }
                  }
              }

              VadState::VoiceActive => {
                  if let Some(mut recognized_voice) = stt::recognize(frame_buffer, false) {
                      info!("Recognized voice: {}", recognized_voice);

                      ipc::send(IpcEvent::SpeechRecognized {
                          text: recognized_voice.clone(),
                      });

                      recognized_voice = recognized_voice.to_lowercase();

                      // Check if wake word was repeated
                      let wake_phrases = config::get_wake_phrases(&i18n::get_language());
                      let contains_wake = wake_phrases.iter().any(|wp| recognized_voice.contains(wp));

                      if contains_wake {
                          let mut remaining = recognized_voice.clone();
                          for wp in &wake_phrases {
                              remaining = remaining.replace(wp.as_str(), "");
                          }
                          let remaining = remaining.trim().to_string();

                          if remaining.is_empty() {
                              if first_recognition {
                                  info!("Discarding initial wake word from prefed audio");
                                  first_recognition = false;
                                  stt::reset_speech_recognizer();
                                  voices::play_reply();
                                  vad_state = VadState::WaitingForVoice;
                                  silence_frames = 0;
                                  start = SystemTime::now();
                                  audio_buffer.clear();
                                  continue;
                              }

                              info!("Wake word repeated during chaining, reactivating...");
                              voices::play_reply();
                              stt::reset_speech_recognizer();
                              ipc::send(IpcEvent::Listening);
                              vad_state = VadState::WaitingForVoice;
                              silence_frames = 0;
                              start = SystemTime::now();
                              audio_buffer.clear();
                              continue;
                          } else {
                              info!("Wake word + command: '{}'", remaining);
                              recognized_voice = remaining;
                          }
                      }

                      first_recognition = false;

                      for tbr in config::get_phrases_to_remove(&i18n::get_language()) {
                          recognized_voice = recognized_voice.replace(tbr, "");
                      }
                      recognized_voice = recognized_voice.trim().to_string();

                      // FIX: use char count (not byte length) — Cyrillic = 2 bytes per char
                      if recognized_voice.chars().count() < 3 {
                          debug!("Ignoring too short recognition: '{}'", recognized_voice);
                          continue;
                      }

                      if recognized_voice.is_empty() {
                          continue;
                      }

                      let should_chain = execute_command(&recognized_voice, rt);

                      if should_chain {
                          info!("Chaining enabled, continuing to listen...");
                          stt::reset_speech_recognizer();
                          vad_state = VadState::WaitingForVoice;
                          silence_frames = 0;
                          start = SystemTime::now();
                          audio_buffer.clear();
                          ipc::send(IpcEvent::Listening);
                          continue;
                      } else {
                          info!("No chain, returning to wake word mode.");
                          return;
                      }
                  }

                  if processed.is_voice {
                      silence_frames = 0;
                  } else {
                      silence_frames += 1;
                      if silence_frames > silence_threshold {
                          // FIX: reset start timer when transitioning back to WaitingForVoice
                          info!("Long silence detected, returning to wake word mode.");
                          vad_state = VadState::WaitingForVoice;
                          silence_frames = 0;
                          start = SystemTime::now();
                          audio_buffer.clear();
                          stt::reset_speech_recognizer();
                      }
                  }
              }
          }

          // Global command timeout
          if let Ok(elapsed) = start.elapsed() {
              if elapsed > config::CMS_WAIT_DELAY {
                  info!("Command timeout, returning to wake word mode.");
                  return;
              }
          }
      }
  }


  fn process_text_command(text: &str, rt: &tokio::runtime::Runtime) {
      info!("Processing text command: {}", text);
      ipc::send(IpcEvent::SpeechRecognized { text: text.to_string() });

      let mut filtered = text.to_lowercase();
      for tbr in config::get_phrases_to_remove(&i18n::get_language()) {
          filtered = filtered.replace(tbr, "");
      }
      let filtered = filtered.trim().to_string();

      if filtered.is_empty() {
          ipc::send(IpcEvent::Idle);
          return;
      }

      execute_command(&filtered, rt);
  }


  fn execute_command(text: &str, rt: &tokio::runtime::Runtime) -> bool {
      let commands_list = COMMANDS_LIST.read();

      if commands_list.is_empty() {
          ipc::send(IpcEvent::Error { message: "Commands not loaded".to_string() });
          ipc::send(IpcEvent::Idle);
          return false;
      }

      let cmd_result = if let Some((intent_id, confidence)) =
          rt.block_on(intent::classify(text))
      {
          info!("Intent recognized: {} (confidence: {:.2})", intent_id, confidence);
          intent::get_command_by_intent(&commands_list, &intent_id)
      } else {
          info!("Intent not recognized, trying fuzzy fallback...");
          commands::fetch_command(text, &commands_list)
      };

      if let Some((cmd_path, cmd_config)) = cmd_result {
          info!("Command found: {:?}", cmd_path);

          let extracted_slots = if !cmd_config.slots.is_empty() {
              let s = slots::extract(text, &cmd_config.slots);
              if !s.is_empty() {
                  info!("Extracted slots: {:?}", s);
                  Some(s)
              } else {
                  None
              }
          } else {
              None
          };

          let lang = i18n::get_language();

          // Play command-specific sound (if defined)
          let sounds = cmd_config.get_sounds(&lang);
          if !sounds.is_empty() {
              voices::play_sounds(&sounds, cmd_path);
          } else {
              voices::play_reply();
          }

          match commands::execute_command(cmd_path, cmd_config, Some(text), extracted_slots.as_ref()) {
              Ok(chain) => {
                  ipc::send(IpcEvent::CommandExecuted {
                      id: cmd_config.id.clone(),
                      success: true,
                  });
                  let should_chain = chain && cmd_config.chain;
                  if !should_chain {
                      ipc::send(IpcEvent::Idle);
                  }
                  should_chain
              }
              Err(e) => {
                  error!("Command execution failed: {}", e);
                  ipc::send(IpcEvent::CommandExecuted {
                      id: cmd_config.id.clone(),
                      success: false,
                  });
                  ipc::send(IpcEvent::Error { message: e });
                  ipc::send(IpcEvent::Idle);
                  false
              }
          }
      } else {
          info!("No matching command found for: '{}'", text);
          ipc::send(IpcEvent::Idle);
          false
      }
  }
  