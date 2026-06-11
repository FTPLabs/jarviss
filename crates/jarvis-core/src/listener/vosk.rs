use crate::{config, stt, i18n};

  pub fn init() -> Result<(), ()> {
      Ok(()) // nothing to init for Vosk
  }

  pub fn data_callback(frame_buffer: &[i16]) -> Option<i32> {
      if let Some((recognized, _confidence)) = stt::recognize_wake_word(frame_buffer) {
          let recognized = recognized.trim().to_lowercase();

          if recognized.is_empty() || recognized == "[unk]" {
              return None;
          }

          info!("Wake word candidate: '{}'", recognized);

          let lang = i18n::get_language();
          let wake_phrases = config::get_wake_phrases(&lang);

          for word in recognized.split_whitespace() {
              if word == "[unk]" {
                  continue;
              }

              let word_chars: Vec<char> = word.chars().collect();

              for wake_phrase in wake_phrases {
                  let wake_chars: Vec<char> = wake_phrase.chars().collect();
                  let similarity = seqdiff::ratio(&wake_chars, &word_chars);

                  if similarity >= config::VOSK_MIN_RATIO {
                      info!("Wake word match: '{}' ~ '{}' ({:.1}%)", word, wake_phrase, similarity);
                      return Some(0);
                  }
              }
          }
      }

      None
  }
  