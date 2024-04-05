use std::process::{Command, Stdio};

use anyhow::Context;

pub struct BuildHints {
    // Can be "dev" or "release", or any custom profile
    pub profile: String,

    // Can be "None", "debuginfo", "symbols", "true" or any invalid value
    strip: Option<String>,
}

impl BuildHints {
    pub fn strip_symbols(&self) -> bool {
        [Some("symbols"), Some("true")].contains(&self.strip.as_deref())
    }
}

#[derive(serde::Deserialize)]
struct UnitGraph {
    units: Vec<Unit>,
}

#[derive(serde::Deserialize)]
struct Unit {
    profile: Profile,
}

#[derive(serde::Deserialize)]
struct Profile {
    name: String,
    strip: Strip,
}

#[derive(serde::Deserialize)]
struct Strip {
    resolved: Option<StripResolved>,
}

#[derive(serde::Deserialize)]
struct StripResolved {
    #[serde(rename = "Named")]
    named: Option<String>,
}

pub fn try_parse_unit_graph(mut command: Command) -> anyhow::Result<BuildHints> {
    command.args(["-Z", "unstable-options", "--unit-graph"]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let stdout = command
        .output()
        .context("Unable to spawn build process")?
        .stdout;
    let json = serde_json::from_slice::<UnitGraph>(&stdout).context("Unable to parse json")?;

    let last_unit = json
        .units
        .into_iter()
        .next_back()
        .context("No units found")?
        .profile;

    Ok(BuildHints {
        profile: last_unit.name,
        strip: last_unit.strip.resolved.and_then(|s| s.named),
    })
}
