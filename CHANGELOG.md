# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.11] - 2024-01-12
### Details
#### Changed
- Replaced unmaintained ftp crate with its fork suppaftp (#8)
- Use log crate for cargo-vita output (#7)
- Disabled no-log feature of suppaftp
- Added logs subcommand to edit PrincessLog config remotely (#10)
- Add rust-src component to recommended rust-toolchain.toml (#11)


## [0.1.11] - 2024-01-12
### Details
#### Changed
- Replace vita-rust wiki link with book link (#5)
- Reverted set_cargo_config_env (#6)


## [0.1.10] - 2023-10-14
### Details
#### Changed
- Fixed release changelog generation
- Passing env variables for pkgconfig during cargo build (#3)


## [0.1.9] - 2023-10-08
### Details
#### Changed
- Return exit code 1 on failure


## [0.1.8] - 2023-10-08
### Details
#### Changed
- Update README.md
- Fail build if cargo build does not succeed


## [0.1.7] - 2023-09-26
### Details
#### Changed
- Auto set OPENSSL_LIB_DIR and OPENSSL_INCLUDE_DIR env vars


## [0.1.6] - 2023-09-13
### Details
#### Changed
- Fixed target path for coredumps


## [0.1.5] - 2023-09-13
### Details
#### Changed
- More build targets
- Fixed coredump parse


## [0.1.4] - 2023-09-13
### Details
#### Changed
- Add cargo env variables (#1)
- Fixed CI pipeline and updated README
- Release v0.1.4


## [0.1.3] - 2023-09-12
### Details
#### Changed
- Added quiet flag as opposed to always having to set -v flag
- Fixed README


## [0.1.2] - 2023-09-12
### Details
#### Changed
- Updated to latest cargo_metadata
- An env fallback for default_title_id
- Update README.md
- Fix static file paths


## [0.1.1] - 2023-09-11
### Details
#### Changed
- VITA_IP is not required for build anymore


## [0.1.0] - 2023-09-11
### Details
#### Changed
- Initial commit
- Initial build implementation
- Build command
- Update and run eboot
- Update and upload flags for vpk
- Readme
- Coredump
- Updated Readme.md
- Release ci job
- Workspace_default_packages hack until cargo_metadata is released
- Updated README.md
- Fixed CI
- Another attempt to fix CI
- Disabled cross-compilation
- Less targets


