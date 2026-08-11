use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::color::ColorConfig;
use super::{Switch, keys};
use crate::t;
use crate::util::{NumSys, UnitStr};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiMode {
	#[default]
	Auto,
	Plain,
	Tui,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigFile {
	#[serde(rename = "Orbit", default, alias = "orbit")]
	pub orbit: OrbitConfig,

	#[serde(rename = "Ui", default, alias = "ui")]
	pub ui: UiConfig,

	#[serde(rename = "Color", default, alias = "color")]
	pub color: ColorConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrbitConfig {
	#[serde(default = "default_true")]
	pub auto_remove: bool,

	#[serde(default = "default_true")]
	pub auto_update: bool,

	#[serde(default)]
	pub update_show_packages: bool,

	#[serde(default)]
	pub simple: bool,

	#[serde(default)]
	pub assume_yes: bool,
}

impl Default for OrbitConfig {
	fn default() -> Self {
		Self {
			auto_remove: true,
			auto_update: true,
			update_show_packages: false,
			simple: false,
			assume_yes: false,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiConfig {
	#[serde(default)]
	pub mode: UiMode,

	#[serde(default)]
	pub unit: NumSys,
}

impl Default for UiConfig {
	fn default() -> Self {
		Self {
			mode: UiMode::Auto,
			unit: NumSys::Binary,
		}
	}
}

fn default_true() -> bool { true }

impl ConfigFile {
	pub fn read(conf_file: &Path) -> Result<Self> {
		let conf = fs::read_to_string(conf_file).with_context(|| {
			t!(
				"file-read-defaults",
				"path" => conf_file.display().to_string()
			)
		})?;

		Self::parse(&conf).with_context(|| {
			t!(
				"file-parse-defaults",
				"path" => conf_file.display().to_string()
			)
		})
	}

	pub fn bool(&self, key: &str) -> Option<bool> {
		match key {
			keys::ASSUME_YES => Some(self.orbit.assume_yes),
			keys::AUTO_REMOVE => Some(self.orbit.auto_remove),
			keys::AUTO_UPDATE => Some(self.orbit.auto_update),
			keys::SIMPLE => Some(self.orbit.simple),
			keys::UPDATE_SHOW_PACKAGES => Some(self.orbit.update_show_packages),
			_ => None,
		}
	}

	pub fn color_mode(&self) -> Switch { self.color.mode }

	pub fn unit_format(&self) -> UnitStr { UnitStr::new(0, self.ui.unit) }

	fn parse(conf: &str) -> Result<Self> { Ok(hcl::from_str(conf)?) }
}

#[cfg(test)]
mod tests {
	use super::{ConfigFile, UiMode};
	use crate::config::Switch;
	use crate::config::color::{ColorCode, Modifiers};
	use crate::util::NumSys;

	#[test]
	fn parses_target_hcl_shape() {
		let conf = include_str!("../../orbit.conf");
		let file = ConfigFile::parse(conf).unwrap();
		assert!(file.orbit.auto_remove);
		assert!(file.orbit.auto_update);
		assert!(!file.orbit.update_show_packages);
		assert!(!file.orbit.simple);
		assert!(!file.orbit.assume_yes);
		assert_eq!(file.ui.mode, UiMode::Auto);
		assert_eq!(file.ui.unit, NumSys::Binary);
		assert_eq!(file.color.mode, Switch::Auto);
		assert_eq!(file.color.theme.primary.fg, ColorCode::LightGreen);
		assert!(file.color.theme.primary.modifier.contains(Modifiers::BOLD));
		assert_eq!(file.color.theme.error.fg, ColorCode::LightRed);
	}

	#[test]
	fn missing_sections_use_defaults() {
		let file = ConfigFile::parse("Ui = { mode = \"Plain\" }").unwrap();
		assert_eq!(file.ui.mode, UiMode::Plain);
		assert!(file.orbit.auto_remove);
		assert_eq!(file.color.mode, Switch::Auto);
	}
}
