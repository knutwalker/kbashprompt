use super::{ModuleInfo, ModuleOut};
use colorful::Style;
use macsmc::Smc;
use std::fmt::Write;

pub(super) fn info() -> ModuleOut {
    let mut smc = Smc::connect().ok()?;
    let battery_info = smc.battery_info().ok()?;

    let (charging, battery_icon) = match (
        battery_info.health_ok,
        battery_info.ac_present,
        battery_info.charging,
        battery_info.battery_powered,
    ) {
        (false, ..) => return ModuleInfo::of().icon("💥").some(),
        (_, true, true, _) => (true, "⚡️"),
        (_, false, _, true) => (false, "🔋"),
        _ => return None,
    };

    const BATTERY_PRECISION: f32 = 1.0;

    for battery in smc.battery_details().ok()? {
        let battery = battery.ok()?;
        let battery_charge =
            ((battery.percentage() / BATTERY_PRECISION).ceil() * BATTERY_PRECISION) as u8;
        if battery_charge < 100 {
            let mut info = String::with_capacity(16);
            write!(info, "{}%", battery_charge).unwrap();

            let time = if charging {
                battery.time_until_full()
            } else {
                battery.time_remaining()
            };

            if let Some(time) = time {
                let secs = time.as_secs();
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                let secs = secs % 60;
                if hours > 0 {
                    write!(info, " {}h", hours).unwrap();
                }
                write!(info, " {:02}m {:02}s", mins, secs).unwrap();
            }

            return ModuleInfo::of()
                .icon(battery_icon)
                .info(info)
                .style(Style::Dim)
                .some();
        }
    }

    None
}
