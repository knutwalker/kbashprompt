use super::{ModuleInfo, ModuleOut};
use libc::{c_double, c_int};

pub(super) fn info() -> ModuleOut {
    let cpus = num_cpus::get_physical();
    let load = load_average_one();
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

fn load_average_one() -> f64 {
    let mut load = 0.0;
    if unsafe { getloadavg(&mut load as _, 1) } != 1 {
        return 0.0;
    }
    load
}

#[link(name = "c")]
extern "C" {
    fn getloadavg(load_avg: *mut c_double, num_elem: c_int) -> c_int;
}
