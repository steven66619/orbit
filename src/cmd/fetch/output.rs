use std::fs;

use anyhow::Result;
use reqwest::Client;
use tokio::task::JoinSet;
use tokio::time::Duration;

use crate::config::{Config, Paths};
use crate::debug;
use crate::t;

async fn check_non_free(
	config: &Config,
	chosen: &[String],
	mut component: String,
	release: &str,
) -> Result<String> {
	let mut set = JoinSet::new();

	if !config.get_bool("non_free", false) {
		return Ok(component);
	}

	let client = Client::builder()
		.connect_timeout(Duration::from_secs(1))
		.timeout(Duration::from_secs(1))
		.build()?;

	for url in chosen.iter() {
		set.spawn(
			client
				.get(format!("{url}/dists/{release}/non-free-firmware/"))
				.send(),
		);
	}

	let mut values = Vec::with_capacity(set.len());
	// Run all of the futures.
	while let Some(res) = set.join_next().await {
		values.push(res.is_ok_and(|r| r.is_ok_and(|r| r.error_for_status().is_ok())));
	}

	// Debatable that we should add separate entries if it exists or not
	if values.iter().all(|b| *b) {
		component += " non-free-firmware";
		return Ok(component);
	}
	Ok(component)
}

pub(super) async fn write_orbit_sources(
	config: &Config,
	chosen: &[String],
	component: String,
	release: &str,
	keyring: &str,
) -> Result<()> {
	debug!("Building Orbit sources file");
	let mut orbit_sources = "# Sources file built for orbit\n\n".to_string();
	// Types: deb deb-src
	// URIs: https://deb.volian.org/volian/
	// Suites: scar
	// Components: main
	// Signed-By: /usr/share/keyrings/volian-archive-scar-unstable.gpg
	orbit_sources += if config.get_bool("sources", false) {
		"Types: deb deb-src\n"
	} else {
		"Types: deb\n"
	};

	orbit_sources += "URIs: ";
	for (i, mirror) in chosen.iter().enumerate() {
		if config.auto().is_some_and(|auto| i + 1 > auto as usize) {
			break;
		}
		if i > 0 {
			orbit_sources += "      ";
		}
		orbit_sources += &format!("{mirror}\n");
	}
	orbit_sources += &format!("Suites: {release}\n");
	orbit_sources += &format!(
		"Components: {}\n",
		check_non_free(config, chosen, component, release).await?
	);
	orbit_sources += &format!("Signed-By: {keyring}\n");

	debug!("Writing the following to file:\n\n{orbit_sources}");

	let file = config.get_file(&Paths::OrbitSources);
	fs::write(&file, orbit_sources)?;
	println!("{}", t!("fetch-sources-written", "file" => file));
	Ok(())
}
