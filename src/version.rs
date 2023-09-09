use std::process;

use rustc_version::Channel;

pub fn check_rust_version() {
    let rust_version = rustc_version::version_meta().unwrap();

    if rust_version.channel > Channel::Nightly {
        eprintln!("cargo-vita requires a nightly rustc version.\n");
        eprintln!(
            "Do one of the following:\n \
            - Run `rustup override set nightly` to use nightly in the current directory\n \
            - Run cargo with +nightly flag.\n \
            - Create a rust-toolchain.toml in the root with the following content:\n   \
              [toolchain]\n   \
              channel = \"nightly\""
        );
        process::exit(1);
    }
}
