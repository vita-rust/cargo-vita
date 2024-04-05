# cargo-vita

[![Crates.io](https://img.shields.io/crates/v/cargo-vita.svg)](https://crates.io/crates/cargo-vita)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/vita-rust/cargo-vita#license)


Cargo command to work with Sony PlayStation Vita rust project binaries.

For general guidelines see the [vita-rust book].

## Requirements

- [VitaSDK] must be installed, and `VITASDK` environment variable must point to its location.
- [vitacompanion] for FTP and command server (uploading and running artifacts)
- [PrincessLog] is required for `cargo vita logs`
- [vita-parse-core] for `cargo vita coredump parse`

## Installation

```
cargo +nightly install cargo-vita
```

## Usage

Use the nightly toolchain to build Vita apps (either by using `rustup override nightly` for the project directory or by adding +nightly in the cargo invocation).


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
vita_make_fself_flags = ["-s"]
# Optional, this is the default
vita_mksfoex_flags = ["-d", "ATTRIBUTE2=12"]

[package.metadata.vita.profile.dev]
# Strips symbols from the vita elf in dev profile. Optional, default is false
strip_symbols = true
[package.metadata.vita.profile.release]
# Strips symbols from the vita elf in release profile. Optional, default is true
strip_symbols = true
```

## Examples

```sh
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

## Additional tools

For a better development experience, it is recommended to install the following modules on your Vita.

### vitacompanion

When enabled, this module keeps an FTP server on your Vita running on port `1337`, as well as a TCP command server running on port `1338`.

- The FTP server allows you to easily upload `vpk` and `eboot` files to your Vita. This FTP server is used by `cargo-vita` for the following commands and flags:

  ```sh
  # Builds a eboot.bin, and uploads it to ux0:/app/TITLEID/eboot.bin
  cargo vita build eboot --update

  # Builds a vpk, and uploads it to ux0:/download/project_name.vpk
  cargo vita build vpk --upload

  # Recursively upload ~/test to ux0:/download
  cargo vita upload -s ~/test -d ux0:/download/
  ```

- The command server allows you to kill and launch applications and reboot your Vita:

  ```sh
  # Reboot your Vita
  cargo vita reboot

  # After uploading the eboot.bin this command will kill the current app,
  # and launch your TITLEID
  cargo vita build eboot --update --run
  ```

### PrincessLog

This module allows capturing stdout and stderr from your Vita.
In order to capture the logs you need to start a TCP server on your computer and configure
PrincessLog to connect to it.

For convenience `cargo-vita` provides two commands to work with logs:

  - A command to start a TCP server Vita will connect to:

    ```sh
    # Start a TCP server on 0.0.0.0, and print all bytes received via the socket to stdout
    cargo vita logs
    ```
  - A command to reconfigure PrincessLog with the new IP/port. This will use
    the FTP server provided by `vitacompanion` to upload a new config.
    If an IP address of your machine is not explicitly provided, it will be guessed
    using [local-ip-address] crate.
    When a configuration file is updated, the changes are not applied until Vita is rebooted.

    ```sh
    # Generate and upload a new config for PrincessLog to your Vita.
    # Will guess a local IP address of the machine where this command is executed.
    # After reconfiguration reboots the Vita.
    cargo vita logs configure && cargo vita reboot


    # Explicitly sets the IP address Vita will connect to.
    # Also enables kernel debug messages in the log.
    cargo vita logs configure --host-ip-address 10.10.10.10 --kernel-debug
    ```

## Notes

To produce the actual artifact runnable on the device, `cargo-vita` does multiple steps:

1. Calls `cargo build` to build the code and link it to a `elf` file (using linker from [VitaSDK])
2. Calls `vita-elf-create` from [VitaSDK] to transform the `elf` into Vita `elf` (`velf`)
3. Calls `vita-make-fself` from [VitaSDK] to sign `velf` into `self` (aka `eboot`).

The second step of this process requires relocation segments in the elf.
This means, that adding `strip=true` or `strip="symbols"` is not supported for Vita target,
since symbol stripping also strips relocation information.

To counter this issue, `cargo-vita` can do an additional strip step of the `elf` with `--strip-unneeded` flag, which reduces the binary size without interfering with other steps necessary to produce a runnable binary.

This step is enabled for release profile builds and disabled for other profile builds by default, but can be configured per-crate via the following section in `Cargo.toml`:

```toml
[package.metadata.vita.profile.dev]
# Strips symbols from the vita elf in dev profile, default is false
strip_symbols = true
[package.metadata.vita.profile.release]
# Strips symbols from the vita elf in release profile, default is true
strip_symbols = true
```


## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed at your option under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))


[vita-rust book]: https://vita-rust.github.io/book
[VitaSDK]: https://vitasdk.org/
[vitacompanion]: https://github.com/devnoname120/vitacompanion
[PrincessLog]: https://github.com/CelesteBlue-dev/PSVita-RE-tools/tree/master/PrincessLog/build
[vita-parse-core]: https://github.com/xyzz/vita-parse-core
[local-ip-address]: https://crates.io/crates/local-ip-address
