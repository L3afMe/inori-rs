# Rust Selfbot using [Serenity-rs](github.com/serenity-rs/serenity/)

## WARNING: Currently in prerelease, there will be features that don't work or are buggy

Originally starting as a personal project to learn Rust, 
Inori-rs is now a fully open source SelfBot available to the public. 

Named after [Inori Yuzuriha](https://guiltycrown.fandom.com/wiki/Inori_Yuzuriha) from Guilty Crown

## How to install

### Prebuild binaries

1) Download the latest version from https://github.com/l3afme/inori-rs/releases/latest
2) Run the file and go through the config setup walkthrough

### Building from source

Prerequisites
- Rust Nightly (https://rustup.rs/)
  - If using rustup then run `rustup default nightly` to install Nightly
- Cargo - Installed alongside Rust

1) `git clone https://github.com/L3afMe/inori-rs`
2) `cd inori-rs`
3) `cargo build --release`
4) The binaries will be located in `inori-rs/target/releases/` as `inori-rs` on Unix or `inori-rs.exe` on Windows
