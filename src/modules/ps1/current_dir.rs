use super::{ModuleInfo, ModuleOut};
use colorful::Color;
use std::{borrow::Cow, env, ffi::OsString};

const GREEN: Color = Color::Chartreuse4; // 64

pub(super) fn info() -> ModuleOut {
    if let Ok(pwd) = env::current_dir() {
        if let Some(home) = home() {
            if let Ok(pwd) = pwd.strip_prefix(home) {
                let pwd = pwd.to_string_lossy();
                let pwd = if pwd.is_empty() {
                    Cow::Borrowed("~")
                } else {
                    Cow::Owned(format!("~/{}", pwd))
                };
                return Some(ModuleInfo::of().info(pwd).color(GREEN));
            }
        }
        let pwd = pwd.to_string_lossy();
        let pwd = pwd.into_owned();
        return Some(ModuleInfo::of().info(pwd).color(GREEN));
    }
    None
}

fn home() -> Option<OsString> {
    env::var_os("HOME").filter(|h| !h.is_empty())
}
