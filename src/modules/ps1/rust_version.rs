use super::{ModuleInfo, ModuleOut};
use colorful::Color;
use std::{env, path::Path, process::Command};

const ORANGE: Color = Color::DarkOrange3b; // 166

pub(super) fn get(wd: &Path) -> ModuleOut {
    let cargo_toml = wd.join("Cargo.toml");
    let _ = cargo_toml.metadata().ok().filter(|f| f.is_file())?;

    let rustc_version = env::var("RUSTC")
        .map_or_else(|_| Command::new("rustc"), Command::new)
        .arg("--version")
        .output()
        .ok()
        .filter(|c| c.status.success())?;

    let rustc_version = String::from_utf8_lossy(&rustc_version.stdout);
    let rustc_version = rustc_version.split_whitespace().nth(1);

    rustc_version.map(|v| ModuleInfo::of().icon("🦀").text(v).color(ORANGE))
}
