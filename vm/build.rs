use std::env;
use std::error::Error;

type Result<T, E = Box<dyn Error + Send + Sync + 'static>> = core::result::Result<T, E>;

fn main() -> Result<()> {
    // This is to allow nix builds to substitute in tmux and screen.
    let tmux_str = String::from("@tmux@");
    let screen_str = String::from("@screen@");

    let tmux_bin = if tmux_str.starts_with('@') && tmux_str.ends_with('@') {
        env::var("TMUX_PATH")?
    } else {
        tmux_str
    };
    let screen_bin = if screen_str.starts_with('@') && screen_str.ends_with('@') {
        env::var("SCREEN_PATH")?
    } else {
        screen_str
    };

    println!("cargo:rustc-env=TMUX_BIN={}", tmux_bin);
    println!("cargo:rustc-env=SCREEN_BIN={}", screen_bin);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PATH");

    Ok(())
}
