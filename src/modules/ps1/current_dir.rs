use super::{ModuleInfo, ModuleOut};
use colorful::Color;
use std::{borrow::Cow, env, path::Path};

const GREEN: Color = Color::Chartreuse4; // 64

pub(super) fn info() -> ModuleOut {
    let pwd = env::current_dir().ok()?;
    let info = ModuleInfo::of().color(GREEN);

    match home_path(&pwd) {
        Some(path) => info.info(path),
        None => info.info(full_path(&pwd)),
    }
    .some()
}

fn home_path(path: &Path) -> Option<Cow<'static, str>> {
    let home = env::var_os("HOME")?;
    if home.is_empty() {
        return None;
    }
    let path = path.strip_prefix(home).ok()?;
    let path = path.to_string_lossy();
    Some(if path.is_empty() {
        "~".into()
    } else {
        format!("~/{}", path).into()
    })
}

fn full_path(path: &Path) -> String {
    let pwd = path.to_string_lossy();
    pwd.into_owned()
}
