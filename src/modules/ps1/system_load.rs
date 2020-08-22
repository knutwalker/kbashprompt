use super::{ModuleInfo, ModuleOut};
use sysinfo::{System, SystemExt};

pub(super) fn info() -> ModuleOut {
    let system = System::new();
    let cpus = num_cpus::get_physical();
    let load = system.get_load_average().one;
    let load = {
        let factor = load / cpus as f64;
        if factor > 4.0 {
            "😰"
        } else if factor > 3.0 {
            "😥"
        } else if factor > 2.0 {
            "😓"
        } else if factor > 1.0 {
            "😅"
        } else {
            return None;
        }
    };

    ModuleInfo::of().icon(load).some()
}
