use super::{ModuleInfo, ModuleOut};
use colorful::Color;
use home::home_dir;
use std::{borrow::Cow, env};

const GREEN: Color = Color::Chartreuse4; // 64

pub(super) fn info() -> ModuleOut {
    if let Ok(pwd) = env::current_dir() {
        if let Some(home) = home_dir() {
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
