use super::ModuleInfo;
use colorful::Style;
use time::OffsetDateTime;

pub(super) fn info() -> ModuleInfo {
    let now = OffsetDateTime::now_local().time();
    let now = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
    ModuleInfo::of().info(now).style(Style::Dim)
}
