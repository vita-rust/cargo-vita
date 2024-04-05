use std::{fmt::Display, ops::Deref, str::FromStr};

use anyhow::Context;
use cargo_metadata::{camino::Utf8PathBuf, Artifact, MetadataCommand, Package};
use serde::Deserialize;

pub static VITA_TARGET: &str = "armv7-sony-vita-newlibeabihf";

#[derive(Clone, Debug)]
pub struct TitleId(String);

impl<'de> Deserialize<'de> for TitleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Deref for TitleId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for TitleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for TitleId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 9 {
            return Err("Title ID must be 9 characters long".to_string());
        }

        if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("Title ID consist of alpha numeric characters only".to_string());
        }

        if !s
            .chars()
            .next()
            .ok_or("Title ID must not be empty")?
            .is_alphabetic()
        {
            return Err("Title ID must start with an alphabetic character".to_string());
        }

        Ok(Self(s.to_uppercase().to_string()))
    }
}

fn default_build_std() -> String {
    "std,panic_unwind".to_string()
}

fn default_vita_make_fself_flags() -> Vec<String> {
    vec!["-s".to_string()]
}

fn default_vita_mksfoex_flags() -> Vec<String> {
    vec!["-d".to_string(), "ATTRIBUTE2=12".to_string()]
}

#[derive(Deserialize, Debug)]
pub struct PackageMetadata {
    pub title_id: Option<TitleId>,
    pub title_name: Option<String>,
    pub assets: Option<String>,
    #[serde(default = "default_build_std")]
    pub build_std: String,
    #[serde(default = "default_vita_make_fself_flags")]
    pub vita_make_fself_flags: Vec<String>,
    #[serde(default = "default_vita_mksfoex_flags")]
    pub vita_mksfoex_flags: Vec<String>,

    #[serde(default)]
    pub dev: ProfileMetadata,
    #[serde(default)]
    pub release: ProfileMetadata,
}

impl PackageMetadata {
    pub fn strip_symbols(&self, release: bool) -> bool {
        if release {
            self.release.strip_symbols.unwrap_or(true)
        } else {
            self.dev.strip_symbols.unwrap_or(false)
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct ProfileMetadata {
    pub strip_symbols: Option<bool>,
}

impl Default for PackageMetadata {
    fn default() -> Self {
        Self {
            title_id: None,
            title_name: None,
            assets: None,
            build_std: default_build_std(),
            vita_make_fself_flags: default_vita_make_fself_flags(),
            vita_mksfoex_flags: default_vita_mksfoex_flags(),
            release: ProfileMetadata::default(),
            dev: ProfileMetadata::default(),
        }
    }
}

pub fn parse_crate_metadata(
    artifact: Option<&Artifact>,
) -> anyhow::Result<(PackageMetadata, Option<Package>, Utf8PathBuf)> {
    let meta = MetadataCommand::new()
        .exec()
        .context("Failed to get cargo metadata")?;

    let pkg = match artifact {
        Some(artifact) => meta.packages.iter().find(|p| p.id == artifact.package_id),
        None => meta.workspace_default_packages().first().copied(),
    };

    if let Some(pkg) = pkg {
        if let Some(metadata) = pkg.metadata.as_object() {
            if let Some(metadata) = metadata.get("vita") {
                let metadata = serde_json::from_value::<PackageMetadata>(metadata.clone())
                    .context("Unable to deserialize `package.metadata.vita`")?;

                return Ok((metadata, Some(pkg.clone()), meta.target_directory));
            }
        }
    }

    Ok((
        PackageMetadata::default(),
        pkg.cloned(),
        meta.target_directory,
    ))
}
