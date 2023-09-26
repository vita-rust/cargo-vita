# cargo-vita

[![Crates.io](https://img.shields.io/crates/v/cargo-vita.svg)](https://crates.io/crates/cargo-vita)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/vita-rust/cargo-vita#license)


Cargo command to work with Sony PlayStation Vita rust project binaries.

For general guidelines see [vita-rust wiki](https://github.com/vita-rust/std/wiki)

## Requirements

- [VitaSDK](https://vitasdk.org/) must be installed, and `VITASDK` environment variable must point to its location.
- [vitacompanion](https://github.com/devnoname120/vitacompanion) for ftp and command server (uploading and running artifacts)
- [PrincessLog](https://github.com/CelesteBlue-dev/PSVita-RE-tools/tree/master/PrincessLog/build) is required for `cargo vita logs`
- [vita-parse-core](https://github.com/xyzz/vita-parse-core) for `cargo vita coredump parse`

## Installation

```
cargo +nightly install cargo-vita
```

## Usage

Use the nightly toolchain to build Vita apps (either by using rustup override nightly for the project directory or by adding +nightly in the cargo invocation).


```
Cargo wrapper for developing Sony PlayStation Vita homebrew apps

Usage: cargo vita [OPTIONS] <COMMAND>

Commands:
  build     Builds the Rust binary/tests/examples into a VPK or any of the intermediate steps
  upload    Uploads files and directories to the Vita vita ftp
  run       Starts an installed title on the Vita by the title id
  logs      Start a TCP server on this machine, to which Vita can stream logs via PrincessLog
  coredump  Download coredump files from the Vita
  reboot    Reboot the Vita
  help      Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet       By default, verbose level is 1. Setting quiet flag will reduce it by one
  -v, --verbose...  Print the exact commands `cargo-vita` is running. Passing this flag multiple times will enable verbose mode for the rust compiler
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version
```

The `build` command pass-through arguments are passed to cargo build.

## Setting up the environment

`cargo-vita` requires you to set `VITASDK` environment variable. In addition to that, if you are planning on
uploading files, running executables and working with core dumps with this tool, it is recommended to set
`VITA_IP` environment variable instead of passing it to every command as an argument.

You can set these environment variables in your shell configuration (such as `.bashrc`), use [direnv](https://direnv.net/),
and additionally this tool will parse your projects `.cargo/config.toml` for `[env]` section.

## Parameterize your project

`cargo-vita` uses information in `Cargo.toml` to build your vpk.

Add the following section to `Cargo.toml` of your project:

```toml
[package.metadata.vita]
# A unique identifier for your project. 9 chars, alphanumeric.
title_id = "RUSTAPP01"
# A title that will be shown on a bubble. Optional, will take the crate name as the default
title_name = "My application"
# Optional. A path to static files relative to the project.
assets = "static"
# Optional, this is the default
build_std = "std,panic_unwind"
# Optional, this is the default
vita_strip_flags = ["-g"]
# Optional, this is the default
vita_make_fself_flags = ["-s"]
# Optional, this is the default
vita_mksfoex_flags = ["-d", "ATTRIBUTE2=12"]
```

## Examples

```
# Build all current/all workspace projects in release mode as vpk
cargo vita build vpk -- --release

# Build tests of current/all workspace projects in release mode as vpk
cargo vita build vpk -- --release --tests

# Build examples of current/all workspace projects in release mode as vpk and upload vpk files to ux0:/download/
cargo vita build vpk --upload -- --release --examples

# Build a eboot.bin, upload it to Vita and run it. The VPK must already be installed for that to work.
cargo vita build eboot --update --run -- --release

# Start a TCP server and listen for logs. Send a termination signal to stop (e.g. ctrl+c)
cargo vita logs
```

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed at your option under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

