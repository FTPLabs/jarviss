use std::{collections::HashMap, path::PathBuf};
  use serde::{Serialize, Deserialize};

  /// Command execution type — typed enum so typos in command.toml
  /// are caught at parse time instead of silently doing nothing.
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

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct SlotDefinition {
      pub label: String,
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

      // phrases per language: { "ru": ["открой браузер", ...], "en": [...] }
      #[serde(default)]
      pub phrases: HashMap<String, Vec<String>>,

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
  }
  