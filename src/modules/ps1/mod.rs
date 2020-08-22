use std::fmt::{self, Display};

use super::{ModuleInfo, ModuleOut};

mod battery;
mod current_dir;
mod current_time;
mod git_env;
mod system_load;

pub(crate) struct Ps1;

impl Display for Ps1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;

        write!(f, "{}", current_time::info())?;

        if let Some(battery_info) = battery::info() {
            write!(f, " {}", battery_info)?;
        }

        if let Some(current_dir) = current_dir::info() {
            write!(f, " {}", current_dir)?;
        }

        git_env::try_with_each(|info| write!(f, " {}", info))?;

        if let Some(sys_info) = system_load::info() {
            write!(f, " {}", sys_info)?;
        }

        writeln!(f)?;
        write!(f, "∵ ")
    }
}
