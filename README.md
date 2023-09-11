# cargo-vita

Cargo command to work with Sony PlayStation Vita rust project binaries.

For general guidelines see [vita-rust wiki](https://github.com/vita-rust/std-newlib/wiki)

## Requirements

- [VitaSDK](https://vitasdk.org/) must be installed, and `VITASDK` environment variable must point to its location.
- [vitacompanion](https://github.com/devnoname120/vitacompanion) for ftp and command server (uploading and running artifacts)
- [PrincessLog](https://github.com/CelesteBlue-dev/PSVita-RE-tools/tree/master/PrincessLog/build) is required for `cargo vita logs`
- [vita-parse-core](https://github.com/xyzz/vita-parse-core) for `cargo vita coredump parse`

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
  -v, --verbose...  Print the exact commands `cargo-vita` is running. Passing this flag multiple times will enable verbose mode for the rust compiler
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version
```

The `build` command pass-through arguments are passed to cargo build.


## Examples

```
# Build all current/all workspace projects in release mode as vpk
cargo vita -v build vpk -- --release

# Build tests of current/all workspace projects in release mode as vpk
cargo vita -v build vpk -- --release --tests

# Build examples of current/all workspace projects in release mode as vpk and upload vpk files to ux0:/download/
cargo vita -v build vpk --upload -- --release --examples

# Build a eboot.bin, upload it to Vita and run it. The VPK must already be installed for that to work.
cargo vita -v build eboot --update --run -- --release

# Start a TCP server and listen for logs. Send a termination signal to stop (e.g. ctrl+c)
cargo vita -v logs
```
