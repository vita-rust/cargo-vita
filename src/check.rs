use std::{
    collections::HashMap,
    env,
    process::{self, Command, Stdio},
};

use anyhow::{anyhow, Context};
use log::error;
use rustc_version::Channel;

pub fn rust_version() -> anyhow::Result<()> {
    let rust_version = rustc_version::version_meta()?;

    if rust_version.channel > Channel::Nightly {
        error!(
            "cargo-vita requires a nightly rustc version. Do one of the following:\n \
            - Run `rustup override set nightly` to use nightly in the current directory\n \
            - Run cargo with +nightly flag.\n \
            - Create a rust-toolchain.toml in the root with the following content:\n   \
              [toolchain]\n   \
              channel = \"nightly\"\n   \
              components = [ \"rust-src\" ]"
        );
        process::exit(1);
    }

    Ok(())
}

pub fn set_cargo_config_env() -> anyhow::Result<()> {
    let cargo = env::var_os("CARGO");
    let mut child = Command::new(cargo.as_deref().unwrap_or_else(|| "cargo".as_ref()))
        .args(["config", "get", "-Zunstable-options", "--format=json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .context("spawning `cargo config get` process")?;

    let CargoConfig { env } = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("`cargo config get` child process has no stdout"))
        .and_then(|stdout| {
            serde_json::from_reader(stdout)
                .context("failed to deserialize `cargo config get` output")
        })
        .map_err(|e| {
            let _ = child.kill();
            e
        })?;

    let status = child.wait().context("running `cargo config get` command")?;
    anyhow::ensure!(status.success(), "`cargo config get` failed: {status:?}");

    env.iter()
        .filter(|(key, _)| env::var_os(key).is_none())
        .for_each(|(key, value)| env::set_var(key, value));

    Ok(())
}

#[derive(serde::Deserialize)]
struct CargoConfig {
    #[serde(default)]
    env: HashMap<String, String>,
}
