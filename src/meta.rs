use std::str::FromStr;

use cargo_metadata::MetadataCommand;
use serde::Deserialize;

pub static VITA_TARGET: &str = "armv7-sony-vita-newlibeabihf";

#[derive(Clone, Debug)]
pub struct TitleId(pub String);

impl<'de> Deserialize<'de> for TitleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
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

fn default_vita_strip_flags() -> Vec<String> {
    vec!["-g".to_string()]
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
    pub vita_sdk: Option<String>,
    #[serde(default = "default_vita_make_fself_flags")]
    pub vita_strip_flags: Vec<String>,
    #[serde(default = "default_vita_mksfoex_flags")]
    pub vita_make_fself_flags: Vec<String>,
}

impl Default for PackageMetadata {
    fn default() -> Self {
        Self {
            title_id: Default::default(),
            title_name: Default::default(),
            assets: Default::default(),
            build_std: default_build_std(),
            vita_sdk: Default::default(),
            vita_strip_flags: default_vita_strip_flags(),
            vita_make_fself_flags: default_vita_make_fself_flags(),
        }
    }
}

pub fn parse_crate_metadata() -> PackageMetadata {
    let meta = MetadataCommand::new()
        .exec()
        .expect("Failed to get cargo metadata");

    if let Some(pkg) = meta.workspace_default_packages().first() {
        if let Some(metadata) = pkg.metadata.as_object() {
            if let Some(metadata) = metadata.get("vita") {
                let metadata = serde_json::from_value::<PackageMetadata>(metadata.clone())
                    .expect("Unable to deserialize `package.metadata.vita`");

                return metadata;
            }
        }
    }

    Default::default()
}
