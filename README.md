# cargo-vita

Cargo command to work with Sony PlayStation Vita rust project binaries.

## Usage

Use the nightly toolchain to build Vita apps (either by using rustup override nightly for the project directory or by adding +nightly in the cargo invocation).


```
Usage: cargo vita [OPTIONS] <COMMAND>

Commands:
  build   Builds the Rust binary/tests/examples into a VPK or any of the intermediate steps
  upload  Uploads files and directories to the Vita vita ftp
  run     Starts an installed title on the Vita by the title id
  logs    Start a TCP server on this machine, to which Vita can stream logs via PrincessLog
  reboot  Reboot the Vita
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Print the exact commands `cargo-vita` is running. Passing this flag multiple times will enable verbose mode for the rust compiler
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version
```

The `build` command pass-through arguments that will be passed to cargo build.


## Examples

```
# Build all current/all workspace projects in release mode as vpk
cargo vita -v build vpk -- --release

# Build tests of current/all workspace projects in release mode as vpk
cargo vita -v build vpk -- --release --tests

# Build a eboot.bin, upload it to Vita and run it. The VPK must already be installed for that to work.
cargo vita -v build eboot --update --run -- --release

# Start a TCP server and listen for logs. Send a termination signal to stop (e.g. ctrl+c)
cargo vita -v logs
```
