use std::{
    env,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::ftp;
use anyhow::{bail, Context};
use cargo_metadata::{camino::Utf8PathBuf, Artifact, Message, Package};
use clap::{Args, Subcommand};
use colored::Colorize;
use either::Either;
use tee::TeeReader;
use walkdir::WalkDir;

use crate::meta::{parse_crate_metadata, PackageMetadata, TitleId, VITA_TARGET};

use super::{ConnectionArgs, Executor, OptionalConnectionArgs, Run};

#[derive(Args, Debug)]
pub struct Build {
    #[command(subcommand)]
    cmd: BuildCmd,

    /// An alphanumeric string of 9 characters. Used as a fallback in case title_id is not defined in Cargo.toml.
    #[arg(long, env="VITA_DEFAULT_TITLE_ID", value_parser = clap::value_parser!(TitleId))]
    default_title_id: Option<TitleId>,

    /// Pass additional options through to the `cargo` command.
    ///
    /// All arguments after the first `--`, or starting with the first unrecognized
    /// option, will be passed through to `cargo` unmodified.
    #[arg(trailing_var_arg = true)]
    #[arg(allow_hyphen_values = true)]
    #[arg(global = true)]
    #[arg(name = "CARGO_ARGS")]
    build_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum BuildCmd {
    Elf,
    Velf,
    Eboot(Eboot),
    Sfo,
    Vpk(Vpk),
}

#[derive(Args, Debug)]
struct Eboot {
    /// Uploads eboot.bin to ux0:app/{title_id}/eboot.bin
    #[arg(long, default_value = "false")]
    update: bool,
    /// Runs the updated app. If multiple eboot files are updated, only the last one is run.
    #[arg(long, default_value = "false")]
    run: bool,
    #[command(flatten)]
    connection: OptionalConnectionArgs,
}

#[derive(Args, Debug)]
struct Vpk {
    #[command(flatten)]
    eboot: Eboot,
    /// Uploads the vpk files to the destination folder
    #[arg(long, default_value = "false")]
    upload: bool,
    /// A directory on Vita where a file will be saved. Slash in the end indicates that it's a directory.
    #[arg(long, short = 'd', default_value = "ux0:/download/")]
    destination: String,
}

struct BuildContext<'a> {
    command: &'a Build,
    sdk: String,

    verbose: u8,
}

impl<'a> BuildContext<'a> {
    pub fn new(command: &'a Build, verbose: u8) -> anyhow::Result<Self> {
        let sdk = std::env::var("VITASDK");
        let sdk = sdk.or_else(|_| {
            bail!(
                "VITASDK environment variable isn't set. Please install the SDK \
                    from https://vitasdk.org/ and set the VITASDK environment variable."
            )
        })?;

        Ok(Self {
            command,
            sdk,
            verbose,
        })
    }

    fn sdk(&self, path: &str) -> PathBuf {
        Path::new(&self.sdk).join(path)
    }

    fn sdk_binary(&self, binary: &str) -> PathBuf {
        self.sdk("bin").join(binary)
    }
}

#[derive(Debug)]
struct ExecutableArtifact {
    artifact: Artifact,
    meta: PackageMetadata,
    package: Package,

    elf: Utf8PathBuf,
}

impl ExecutableArtifact {
    fn new(artifact: Artifact) -> anyhow::Result<Self> {
        let (meta, package, _) = parse_crate_metadata(Some(&artifact))?;
        let package = package.context("artifact does not have a package")?;

        let executable = artifact
            .executable
            .as_deref()
            .context("Artifact has no executables")?
            .to_owned();

        Ok(Self {
            artifact,
            meta,
            package,
            elf: executable,
        })
    }
}

impl Executor for Build {
    fn execute(&self, verbose: u8) -> anyhow::Result<()> {
        let ctx = BuildContext::new(self, verbose)?;

        match &self.cmd {
            BuildCmd::Elf => {
                ctx.build_elf()?;
            }
            BuildCmd::Velf => {
                for art in ctx.build_elf()? {
                    ctx.strip(&art)?;
                    ctx.velf(&art)?;
                }
            }
            BuildCmd::Eboot(args) => {
                let artifacts = ctx.build_elf()?;

                for art in &artifacts {
                    ctx.strip(art)?;
                    ctx.velf(art)?;
                    ctx.eboot(art)?;
                }

                if args.update {
                    let files = ctx.eboot_uploads(&artifacts)?;
                    ctx.upload(&files, &args.connection.clone().required()?)?;
                }

                if args.run {
                    ctx.run(&artifacts, &args.connection.clone().required()?)?;
                }
            }
            BuildCmd::Sfo => {
                for art in ctx.build_elf()? {
                    ctx.sfo(&art)?;
                }
            }
            BuildCmd::Vpk(args) => {
                let artifacts = ctx.build_elf()?;

                for art in &artifacts {
                    ctx.strip(art)?;
                    ctx.velf(art)?;
                    ctx.eboot(art)?;
                    ctx.sfo(art)?;
                    ctx.vpk(art)?;
                }

                let mut upload_files = Vec::new();

                if args.upload {
                    upload_files.extend(ctx.vpk_uploads(&artifacts, &args.destination)?);
                }

                if args.eboot.update {
                    upload_files.extend(ctx.eboot_uploads(&artifacts)?);
                }

                if !upload_files.is_empty() {
                    ctx.upload(&upload_files, &args.eboot.connection.clone().required()?)?;
                }

                if args.eboot.run {
                    ctx.run(&artifacts, &args.eboot.connection.clone().required()?)?;
                }
            }
        };

        Ok(())
    }
}

impl<'a> BuildContext<'a> {
    fn build_elf(&self) -> anyhow::Result<Vec<ExecutableArtifact>> {
        let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

        let rust_flags = env::var("RUSTFLAGS").unwrap_or_default()
            + " --cfg mio_unsupported_force_poll_poll --cfg mio_unsupported_force_waker_pipe";

        let mut command = Command::new(cargo);

        if let Ok(path) = env::var("PATH") {
            let sdk_path = Path::new(&self.sdk).join("bin");
            let path = format!("{}:{path}", sdk_path.display());
            command.env("PATH", path);
        }

        // FIXME: A horrible solution, the same -Z flag will be used for all of the crates in a workspace.
        let (meta, _, _) = parse_crate_metadata(None)?;

        command
            .env("RUSTFLAGS", rust_flags)
            .env("TARGET_CC", "arm-vita-eabi-gcc")
            .env("TARGET_CXX", "arm-vita-eabi-g++")
            .pass_path_env("OPENSSL_LIB_DIR", || self.sdk("arm-vita-eabi").join("lib"))
            .pass_path_env("OPENSSL_INCLUDE_DIR", || {
                self.sdk("arm-vita-eabi").join("include")
            })
            .pass_path_env("PKG_CONFIG_PATH", || {
                self.sdk("arm-vita-eabi").join("lib").join("pkgconfig")
            })
            .pass_env("PKG_CONFIG_SYSROOT_DIR", || &self.sdk)
            .env("VITASDK", &self.sdk)
            .arg("build")
            .arg("-Z")
            .arg(format!("build-std={}", meta.build_std))
            .arg("--target")
            .arg(VITA_TARGET)
            .arg("--message-format")
            .arg("json-render-diagnostics")
            .args(&self.command.build_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        if self.verbose > 0 {
            println!("{} {command:?}", "Running cargo:".blue());
        }

        let mut process = command.spawn().context("Unable to spawn build process")?;
        let command_stdout = process.stdout.take().context("Build failed")?;

        let reader = if self.verbose > 1 {
            Either::Left(BufReader::new(TeeReader::new(command_stdout, io::stdout())))
        } else {
            Either::Right(BufReader::new(command_stdout))
        };

        let messages: Vec<Message> = Message::parse_stream(reader)
            .collect::<io::Result<_>>()
            .context("Unable to parse build stdout")?;

        let artifacts = messages
            .iter()
            .rev()
            .filter_map(|m| match m {
                Message::CompilerArtifact(art) if art.executable.is_some() => Some(art.clone()),
                _ => None,
            })
            .map(ExecutableArtifact::new)
            .collect::<anyhow::Result<_>>()?;

        if !process.wait_with_output()?.status.success() {
            bail!("cargo build failed")
        }

        Ok(artifacts)
    }

    fn strip(&self, art: &ExecutableArtifact) -> anyhow::Result<()> {
        let mut command = Command::new(self.sdk_binary("arm-vita-eabi-strip"));

        command
            .args(&art.meta.vita_strip_flags)
            .arg(&art.elf)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if self.verbose > 0 {
            println!("{} {command:?}", "Stripping elf:".blue());
        }

        if !command.status()?.success() {
            bail!("arm-vita-eabi-strip failed");
        }

        Ok(())
    }

    fn velf(&self, art: &ExecutableArtifact) -> anyhow::Result<()> {
        let mut command = Command::new(self.sdk_binary("vita-elf-create"));
        let elf = &art.elf;
        let velf = elf.with_extension("velf");

        command
            .arg(elf)
            .arg(velf)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if self.verbose > 0 {
            println!("{} {command:?}", "Creating velf:".blue());
        }

        if !command.status()?.success() {
            bail!("vita-elf-create failed");
        }

        Ok(())
    }

    fn eboot(&self, art: &ExecutableArtifact) -> anyhow::Result<()> {
        let mut command = Command::new(self.sdk_binary("vita-make-fself"));
        let elf = &art.elf;
        let velf = elf.with_extension("velf");
        let eboot = elf.with_extension("self");

        command
            .args(&art.meta.vita_make_fself_flags)
            .arg(&velf)
            .arg(&eboot)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if self.verbose > 0 {
            println!("{} {command:?}", "Creating eboot:".blue());
        }

        if !command.status()?.success() {
            bail!("vita-make-fself failed");
        }

        Ok(())
    }

    fn sfo(&self, art: &ExecutableArtifact) -> anyhow::Result<()> {
        let mut command = Command::new(self.sdk_binary("vita-mksfoex"));
        let elf = &art.elf;
        let sfo = elf.with_extension("sfo");

        let title_name = art
            .meta
            .title_name
            .as_deref()
            .unwrap_or_else(|| &art.package.name);

        let title_id = &art
            .meta
            .title_id
            .as_ref()
            .or(self.command.default_title_id.as_ref())
            .context(format!(
                "title_id is not set for artifact {}",
                art.package.name
            ))?;

        command.args(&art.meta.vita_mksfoex_flags);
        command.arg("-s").arg(format!("TITLE_ID={title_id}"));
        command
            .arg(title_name)
            .arg(sfo)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if self.verbose > 0 {
            println!("{} {command:?}", "Creating sfo:".blue());
        }

        if !command.status()?.success() {
            bail!("vita-mksfoex failed");
        }

        Ok(())
    }

    fn vpk(&self, art: &ExecutableArtifact) -> anyhow::Result<()> {
        let elf = &art.elf;
        let vpk = elf.with_extension("vpk");
        let eboot = elf.with_extension("self");
        let sfo = elf.with_extension("sfo");

        let mut command = Command::new(self.sdk_binary("vita-pack-vpk"));
        command.arg("-s").arg(sfo);
        command.arg("-b").arg(eboot);

        if let Some(assets) = &art.meta.assets {
            let assets = art
                .artifact
                .manifest_path
                .parent()
                .context("Unable to get target manifest directory")?
                .join(assets);

            let files = WalkDir::new(&assets)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for file in files {
                command.arg("--add").arg(format!(
                    "{}={}",
                    file.path().display(), // path on FS
                    file.path()
                        .strip_prefix(&assets)
                        .context("Unable to strip VPK prefix")?
                        .display()  // path in VPK
                ));
            }
        }

        command
            .arg(vpk)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if self.verbose > 0 {
            println!("{} {command:?}", "Building vpk:".blue());
        }

        if !command.status()?.success() {
            bail!("vita-pack-vpk failed")
        }

        Ok(())
    }

    fn vpk_uploads(
        &self,
        artifacts: &[ExecutableArtifact],
        destination: &str,
    ) -> anyhow::Result<Vec<(Utf8PathBuf, String)>> {
        artifacts
            .iter()
            .map(|a| {
                let src = a.elf.with_extension("vpk");

                let separator = if destination.ends_with('/') { "" } else { "/" };
                let dest = format!(
                    "{destination}{separator}{}",
                    src.file_name().unwrap_or_default()
                );

                Ok((src, dest))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn eboot_uploads(
        &self,
        artifacts: &[ExecutableArtifact],
    ) -> anyhow::Result<Vec<(Utf8PathBuf, String)>> {
        artifacts
            .iter()
            .map(|a| {
                let title_id = a
                    .meta
                    .title_id
                    .as_ref()
                    .or(self.command.default_title_id.as_ref())
                    .context("No title_id provided for artifact")?;

                Ok((
                    a.elf.with_extension("self"),
                    format!("ux0:/app/{title_id}/eboot.bin"),
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn upload(&self, files: &[(Utf8PathBuf, String)], conn: &ConnectionArgs) -> anyhow::Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let mut ftp = ftp::connect(conn, self.verbose)?;

        for (src, dest) in files {
            if self.verbose > 0 {
                println!("{} {src} {} {dest}", "Uploading file".blue(), "to".blue())
            }

            let src = File::open(src).context("Unable to open source file")?;
            ftp.put(dest, &mut BufReader::new(src))
                .context("Failed to upload file")?;
        }

        Ok(())
    }

    fn run(&self, artifacts: &[ExecutableArtifact], conn: &ConnectionArgs) -> anyhow::Result<()> {
        if let Some(art) = artifacts.last() {
            let title_id = art
                .meta
                .title_id
                .as_ref()
                .or(self.command.default_title_id.as_ref());

            if let Some(title_id) = title_id {
                Run {
                    title_id: Some(title_id.clone()),
                    connection: conn.clone(),
                }
                .execute(self.verbose)?;
            }
        }

        Ok(())
    }
}

trait CommandExt {
    fn pass_env<K, V>(&mut self, key: K, default: impl Fn() -> V) -> &mut Command
    where
        K: AsRef<str>,
        V: AsRef<str>;

    fn pass_path_env<K, V>(&mut self, key: K, default: impl Fn() -> V) -> &mut Command
    where
        K: AsRef<str>,
        V: AsRef<Path>,
    {
        self.pass_env(key, || default().as_ref().to_string_lossy().to_string())
    }
}

impl CommandExt for Command {
    fn pass_env<K, V>(&mut self, key: K, default: impl Fn() -> V) -> &mut Command
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = key.as_ref();
        match env::var(key) {
            Ok(val) => self.env(key, val),
            Err(_) => self.env(key, default().as_ref()),
        }
    }
}
