pub mod structs;
use structs::AudioType;
use structs::RecorderType;
use structs::SpeechToTextEngine;
use structs::WakeWordEngine;

use once_cell::sync::Lazy;
use std::env;
use std::fs;
use std::path::PathBuf;

use platform_dirs::AppDirs;

#[cfg(feature="jarvis_app")]
use rustpotter::{
    AudioFmt, BandPassConfig, DetectorConfig, FiltersConfig, GainNormalizationConfig,
    RustpotterConfig, ScoreMode,
};

use crate::config::structs::NoiseSuppressionBackend;
use crate::{APP_CONFIG_DIR, APP_DIRS, APP_LOG_DIR};

#[allow(dead_code)]
pub fn init_dirs() -> Result<(), String> {
    // Инициализация директорий приложения
    if APP_DIRS.get().is_some() {
        return Ok(());
    }

    // cache_dir, config_dir, data_dir, state_dir
    APP_DIRS
        .set(AppDirs::new(Some(BUNDLE_IDENTIFIER), false).unwrap())
        .unwrap();

    // Настройка путей конфигурации и логов
    let mut config_dir = PathBuf::from(&APP_DIRS.get().unwrap().config_dir);
    let mut log_dir = PathBuf::from(&APP_DIRS.get().unwrap().config_dir);

    // Создание директорий при необходимости
    if !config_dir.exists() {
        if fs::create_dir_all(&config_dir).is_err() {
            config_dir = env::current_dir().expect("Cannot infer the config directory");
            fs::create_dir_all(&config_dir)
                .expect("Cannot create config directory, access denied?");
        }
    }

    if !log_dir.exists() {
        if fs::create_dir_all(&log_dir).is_err() {
            log_dir = env::current_dir().expect("Cannot infer the log directory");
            fs::create_dir_all(&log_dir).expect("Cannot create log directory, access denied?");
        }
    }

    // Сохранение путей
    APP_CONFIG_DIR.set(config_dir).unwrap();
    APP_LOG_DIR.set(log_dir).unwrap();

    Ok(())
}

/*
   Настройки по умолчанию для FTPDev Voice Assistant.
*/
pub const DEFAULT_AUDIO_TYPE: AudioType = AudioType::Kira;
pub const DEFAULT_RECORDER_TYPE: RecorderType = RecorderType::PvRecorder;
pub const DEFAULT_WAKE_WORD_ENGINE: WakeWordEngine = WakeWordEngine::Vosk;
pub const DEFAULT_SPEECH_TO_TEXT_ENGINE: SpeechToTextEngine = SpeechToTextEngine::Vosk;

// Идентификаторы бэкендов (строковые ID)
pub const DEFAULT_INTENT_BACKEND: &str = "intent-classifier";
pub const DEFAULT_SLOTS_BACKEND: &str = "none";
pub const DEFAULT_VAD_BACKEND: &str = "energy";

pub const DEFAULT_VOICE: &str = "ftpdev-remaster";
pub const SOUND_PATH: &str = "resources/sound";
pub const VOICES_PATH: &str = "voices";

// Идентификаторы бренда FTPDev
pub const BUNDLE_IDENTIFIER: &str = "com.ftpdev.assistant";
pub const DB_FILE_NAME: &str = "app.db";
pub const LOG_FILE_NAME: &str = "log.txt";
pub const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
pub const AUTHOR_NAME: Option<&str> = Some("FTPDev");
pub const REPOSITORY_LINK: Option<&str> = Some("https://github.com/FTPLabs/jarviss");
pub const TG_OFFICIAL_LINK: Option<&str> = None;
pub const FEEDBACK_LINK: Option<&str> = None;
pub const SUPPORT_BOOSTY_LINK: Option<&str> = None;
pub const SUPPORT_PATREON_LINK: Option<&str> = None;

/*
   Трей-иконка.
*/
pub const TRAY_ICON: &str = "32x32.png";
pub const TRAY_TOOLTIP: &str = "FTPDev Voice Assistant";

// RUSTPOTTER
pub const RUSPOTTER_MIN_SCORE: f32 = 0.62;

#[cfg(feature="jarvis_app")]
pub const RUSTPOTTER_DEFAULT_CONFIG: Lazy<RustpotterConfig> = Lazy::new(|| {
    RustpotterConfig {
        fmt: AudioFmt::default(),
        detector: DetectorConfig {
            avg_threshold: 0.,
            threshold: 0.5,
            min_scores: 15,
            score_ref: 0.22,
            band_size: 5,
            vad_mode: None,
            score_mode: ScoreMode::Max,
            eager: false,
        },
        filters: FiltersConfig {
            gain_normalizer: GainNormalizationConfig {
                enabled: false,
                gain_ref: None,
                min_gain: 0.7,
                max_gain: 1.0,
            },
            band_pass: BandPassConfig {
                enabled: true,
                low_cutoff: 80.,
                high_cutoff: 400.,
            },
        },
    }
});

// PICOVOICE
pub const COMMANDS_PATH: &str = "resources/commands/";
pub const KEYWORDS_PATH: &str = "resources/picovoice/keywords/";
pub const DEFAULT_KEYWORD: &str = "ftpdev_windows.ppn";
pub const DEFAULT_SENSITIVITY: f32 = 1.0;

// VOSK
pub const VOSK_MODELS_PATH: &str = "resources/vosk";
pub const VOSK_MODEL_PATH: &str = "resources/vosk/model_small";
pub const VOSK_FETCH_PHRASE: &str = "фтп";
pub const VOSK_MIN_RATIO: f64 = 70.0;

pub const VOSK_WAKE_CONFIDENCE: f32 = 0.9;

pub const VOSK_SPEECH_RECOGNIZER_MAX_ALTERNATIVES: u16 = 3;
pub const VOSK_SPEECH_RECOGNIZER_WORDS: bool = false;
pub const VOSK_SPEECH_PARTIAL_WORDS: bool = false;

// Распознавание намерений
pub const INTENT_CLASSIFIER_MIN_CONFIDENCE: f64 = 0.75;

// Классификатор на основе эмбеддингов
pub const EMBEDDING_MIN_CONFIDENCE: f64 = 0.70;

// Настройки аудио-обработки по умолчанию
pub const DEFAULT_NOISE_SUPPRESSION: NoiseSuppressionBackend = NoiseSuppressionBackend::None;
pub const DEFAULT_GAIN_NORMALIZER: bool = false;

// Настройки VAD
pub const VAD_ENERGY_THRESHOLD: f32 = 100.0;
pub const VAD_NNNOISELESS_THRESHOLD: f32 = 0.8;
pub const VAD_SILENCE_FRAMES: u32 = 15;

// Настройки нормализации громкости
pub const GAIN_TARGET_RMS: f32 = 3000.0;
pub const GAIN_MIN: f32 = 0.5;
pub const GAIN_MAX: f32 = 3.0;

// Размер фрейма nnnoiseless (фиксирован библиотекой)
pub const NNNOISELESS_FRAME_SIZE: usize = 480;

// LUA
pub const DEFAULT_LUA_SANDBOX: &str = "standard";
pub const DEFAULT_LUA_TIMEOUT: u64 = 10000; // мс

// Прочее
pub const CMD_RATIO_THRESHOLD: f64 = 75f64;
pub const CMS_WAIT_DELAY: std::time::Duration = std::time::Duration::from_secs(15);

/// Возвращает список фраз активации для заданного языка.
pub fn get_wake_phrases(lang: &str) -> &'static [&'static str] {
    match lang {
        "ru" => &["эф ти пи", "фтп", "ассистент"],
        "ua" => &["еф ті пі", "фтп", "асистент"],
        "en" => &["ftp", "assistant", "dev"],
        _ => &["ftp"],
    }
}

/// Возвращает список фраз, которые нужно удалить из распознанного текста.
pub fn get_phrases_to_remove(lang: &str) -> &'static [&'static str] {
    match lang {
        "ru" => &[
            "эф ти пи", "фтп", "ассистент",
            "сэр", "скажи", "покажи", "давай",
        ],
        "ua" => &[
            "еф ті пі", "фтп", "асистент",
            "сер", "скажи", "покажи", "давай",
        ],
        "en" => &[
            "ftp", "assistant", "dev",
            "please", "say", "show",
        ],
        _ => &["ftp"],
    }
}

/// Возвращает грамматику для Vosk wake-word.
pub fn get_wake_grammar(lang: &str) -> &'static [&'static str] {
    match lang {
        "ru" => &[
            "фтп", "ассистент", "[unk]",
            "привет", "давай",
        ],
        "ua" => &[
            "фтп", "асистент", "[unk]",
            "привіт", "давай",
        ],
        "en" => &[
            "ftp", "assistant", "dev", "[unk]",
            "hello", "hey", "hi",
        ],
        _ => &["ftp", "[unk]"],
    }
}
