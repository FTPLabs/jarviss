use std::{collections::HashMap, path::PathBuf};
use serde::{Serialize, Deserialize};
use std::fmt;

/// Command execution type — typed enum so typos in command.toml are caught at parse time.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    Lua,
    Ahk,
    Cli,
    Voice,
    Terminate,
    StopChaining,
}

impl Default for CommandType {
    fn default() -> Self { CommandType::Lua }
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotDefinition {
    pub label: String,
    /// GLiNER entity type to extract for this slot (e.g. "person", "city")
    #[serde(default)]
    pub entity: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum SlotValue {
    Text(String),
    Number(f64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JCommandsList {
    #[serde(skip)]
    pub path: PathBuf,
    pub commands: Vec<JCommand>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JCommand {
    pub id: String,

    #[serde(rename = "type")]
    pub cmd_type: CommandType,

    #[serde(default)]
    pub description: String,

    // for "ahk" type
    #[serde(default)]
    pub exe_path: String,
    #[serde(default)]
    pub exe_args: Vec<String>,

    // for "cli" type
    #[serde(default)]
    pub cli_cmd: String,
    #[serde(default)]
    pub cli_args: Vec<String>,

    // for "lua" type
    #[serde(default)]
    pub script: String,

    // Lua sandbox level: "minimal", "standard", "full"
    #[serde(default)]
    pub sandbox: String,

    // execution timeout in ms (0 = use default)
    #[serde(default)]
    pub timeout: u64,

    // phrases per language: { "ru": ["открой браузер", ...], "en": [...] }
    #[serde(default)]
    pub phrases: HashMap<String, Vec<String>>,

    // per-language sound file names to play on successful execution
    // e.g. { "ru": ["ok.mp3"], "en": ["done.wav"] }
    #[serde(default)]
    pub sounds: HashMap<String, Vec<String>>,

    // slot definitions: slot_name -> SlotDefinition
    #[serde(default)]
    pub slots: HashMap<String, SlotDefinition>,

    // whether to chain to next command after this one
    #[serde(default = "default_chain")]
    pub chain: bool,
}

fn default_chain() -> bool { true }

impl JCommand {
    pub fn get_phrases<'a>(&'a self, lang: &str) -> Vec<&'a str> {
        self.phrases
            .get(lang)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Sound files for this command in the given language.
    /// Returns an owned Vec<String> so callers can pass it as &[String] to voices.
    pub fn get_sounds(&self, lang: &str) -> Vec<String> {
        self.sounds
            .get(lang)
            .cloned()
            .unwrap_or_default()
    }

    /// Effective timeout in ms — fallback to 10s if not set in toml.
    pub fn effective_timeout_ms(&self) -> u64 {
        if self.timeout == 0 { 10_000 } else { self.timeout }
    }
}
