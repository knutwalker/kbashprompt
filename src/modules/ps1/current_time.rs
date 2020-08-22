use super::ModuleInfo;
use chrono::Local;
use colorful::Style;

pub(super) fn info() -> ModuleInfo {
    ModuleInfo::of()
        .text(Local::now().format("%H:%M:%S"))
        .style(Style::Dim)
}
