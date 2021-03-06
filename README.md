# Due to a bug in Serenity which causes Inori-rs to block for some users after a few hours, development has ceased and everything is being migrated to Golang at [NekoGo](https://github.com/L3afMe/NekoGo) If anyone is able to reliably recreate and/or fix this issue then Inori development will continue

# Inori-rs - A Rust Selfbot using [Serenity-rs](https://github.com/serenity-rs/serenity/)
![License](https://img.shields.io/github/license/L3afMe/Inori-rs?color=%23FAB1ED&style=for-the-badge)
![Latest release](https://img.shields.io/github/v/release/L3afMe/inori-rs?color=FAB1ED&include_prereleases&sort=semver&style=for-the-badge)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/L3afMe/Inori-rs/Rust%20Checker?color=%23FAB1ED&style=for-the-badge)
![Lines of code](https://img.shields.io/tokei/lines/github/L3afMe/inori-rs?color=%23FAB1ED&style=for-the-badge)

## WARNING: Currently in prerelease, there will be features that don't work or are buggy

Originally starting as a personal project to learn Rust, 
Inori-rs is now a fully open source SelfBot available to the public. 

Named after [Inori Yuzuriha](https://guiltycrown.fandom.com/wiki/Inori_Yuzuriha) from Guilty Crown

## How to install

### Prebuild binaries

1) Download the latest version from https://github.com/l3afme/inori-rs/releases/latest
2) Run the file and go through the config setup walkthrough
  - Linux requires `build-essential` and `glibc_2.32`

### Building from source

Prerequisites
- Rust nightly (https://rustup.rs/)
  - If using rustup then run `rustup default nightly` to install Nightly
- Cargo - Installed alongside Rust

How to build
1) `git clone https://github.com/L3afMe/inori-rs`
2) `cd inori-rs`
3) `cargo build --release`
4) The binaries will be located in `inori-rs/target/releases/` as `inori-rs` on Unix or `inori-rs.exe` on Windows

## Contributing

Pull requests for bug fixes and new features are more than welcomed but please ensure that you have the latest RustFmt (built from source) installed and have formatted your code before submitting.

P.S. PRs for spelling and grammar mistakes are more than welcome!

## Command List

There is a list of commands in [COMMANDS.md](COMMANDS.md) at the root of the project, please note that this list may not be 100% up to date. Run `help` to get an up to date list.
