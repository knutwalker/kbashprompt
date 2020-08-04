use chrono::Local;
use colorful::{Color, Colorful, Style};
use git2::{DescribeOptions, ReferenceType, Repository};
use home::home_dir;
use macsmc::Smc;
use std::{
    env::{self, args},
    fmt::{self, Display, Write as FmtWrite},
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::Command,
};
use sysinfo::{System, SystemExt};

const _WHITE: Color = Color::White; // 15
const _BLUE: Color = Color::DodgerBlue1; // 33
const CYAN: Color = Color::LightSeaGreen; // 37
const VIOLET: Color = Color::SlateBlue3a; // 61
const GREEN: Color = Color::Chartreuse4; // 64
const _RED: Color = Color::Red3a; // 124
const PURPLE: Color = Color::DeepPink4c; // 125
const YELLOW: Color = Color::DarkGoldenrod; // 136
const ORANGE: Color = Color::DarkOrange3b; // 166

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    if args().count() <= 1 {
        let _ = write!(stdout, "{}", Ps1);
    } else {
        let _ = write!(stdout, "{}", Ps2);
    }
}

#[cfg(debug_assertions)]
macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Ok(thing) => thing,
            Err(e) => {
                eprintln!(
                    "[{}:{}] !! {} !! = {:#?}",
                    file!(),
                    line!(),
                    stringify!($e),
                    e
                );
                return None;
            }
        };
    };
}

#[cfg(not(debug_assertions))]
macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Ok(thing) => thing,
            Err(_) => return None,
        };
    };
}

#[cfg(debug_assertions)]
macro_rules! map_err {
    ($e:expr) => {
        match $e {
            Ok(thing) => thing,
            Err(e) => {
                eprintln!(
                    "[{}:{}] !! {} !! = {:#?}",
                    file!(),
                    line!(),
                    stringify!($e),
                    e
                );
                return Err(::std::fmt::Error);
            }
        };
    };
}

#[cfg(not(debug_assertions))]
macro_rules! map_err {
    ($e:expr) => {
        match $e {
            Ok(thing) => thing,
            Err(_) => return Err(::std::fmt::Error),
        };
    };
}

struct Ps1;

impl Display for Ps1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut smc = map_err!(Smc::connect());

        writeln!(f)?;
        current_time(f)?;
        battery_info(&mut smc, f)?;
        current_dir(f)?;
        git_prompt(f)?;
        sys_info(&mut smc, f)?;
        writeln!(f)?;
        write!(f, "∵ ")
    }
}

struct Ps2;

impl Display for Ps2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "→ ".color(YELLOW))
    }
}

fn current_time(f: &mut fmt::Formatter) -> fmt::Result {
    write!(
        f,
        "{}",
        Local::now()
            .format("%H:%M:%S")
            .to_string()
            .style(Style::Dim)
    )
}

fn battery_info(smc: &mut Smc, f: &mut fmt::Formatter) -> fmt::Result {
    let battery_info = map_err!(smc.battery_info());

    let (charging, battery_icon) = match (
        battery_info.health_ok,
        battery_info.ac_present,
        battery_info.charging,
        battery_info.battery_powered,
    ) {
        (false, ..) => return write!(f, " 💥"),
        (_, true, true, _) => (true, "⚡️"),
        (_, false, _, true) => (false, "🔋"),
        _ => return Ok(()),
    };

    const BATTERY_PRECISION: f32 = 1.0;

    for battery in map_err!(smc.battery_details()) {
        let battery = map_err!(battery);
        let battery_charge =
            ((battery.percentage() / BATTERY_PRECISION).ceil() * BATTERY_PRECISION) as u8;
        if battery_charge < 100 {
            let mut info = format!("{}%", battery_charge);

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

            write!(f, " {} {}", battery_icon, info.style(Style::Dim))?
        }
    }

    Ok(())
}

fn current_dir(f: &mut fmt::Formatter) -> fmt::Result {
    if let Ok(pwd) = env::current_dir() {
        if let Some(home) = home_dir() {
            if let Ok(pwd) = pwd.strip_prefix(home) {
                let pwd = pwd.to_string_lossy();
                if pwd.is_empty() {
                    write!(f, " {}", "~".color(GREEN))?;
                } else {
                    write!(f, " {}{}", "~/".color(GREEN), pwd.color(GREEN))?;
                }
                return Ok(());
            }
        }
        write!(f, " {}", pwd.to_string_lossy().color(GREEN))?
    }
    Ok(())
}

fn git_prompt(f: &mut fmt::Formatter) -> fmt::Result {
    if let Ok(repo) = Repository::open_from_env() {
        repo_type(&repo, f).unwrap_or(Ok(()))?;
        branch_name(&repo, f)?;
        repo_state(&repo, f)?;
    }
    Ok(())
}

fn repo_type(repo: &Repository, f: &mut fmt::Formatter) -> Option<fmt::Result> {
    let wd = repo.workdir()?;
    if let Some(Err(e)) = rust_version(wd, f) {
        return Some(Err(e));
    }
    if let Some(Err(e)) = java_version(wd, f) {
        return Some(Err(e));
    }

    Some(Ok(()))
}

fn rust_version(wd: impl AsRef<Path>, f: &mut fmt::Formatter) -> Option<fmt::Result> {
    let cargo_toml = wd.as_ref().join("Cargo.toml");
    let cargo_toml = cargo_toml.metadata().ok()?;
    if !cargo_toml.is_file() {
        return None;
    }

    let rustc_version = unwrap!(Command::new("rustc").arg("--version").output());
    if !rustc_version.status.success() {
        return None;
    }

    let rustc_version = unwrap!(String::from_utf8(rustc_version.stdout));
    let rustc_version = rustc_version.split_whitespace().nth(1)?;

    Some(write!(f, " 🦀 {}", rustc_version.color(ORANGE)))
}

fn java_version(wd: impl AsRef<Path>, f: &mut fmt::Formatter) -> Option<fmt::Result> {
    let has_file = move |file: &str| -> bool {
        let build_file = wd.as_ref().join(file);
        build_file.metadata().ok().filter(|f| f.is_file()).is_some()
    };

    for file in [
        "build.gradle",
        "pom.xml",
        "build.gradle.kts",
        "build.sbt",
        "build.xml",
        ".java-version",
    ]
    .iter()
    {
        if has_file(file) {
            let java_home = env::var_os("JAVA_HOME")?;
            let java_home = PathBuf::from(java_home);
            let java_home = java_home.read_link().unwrap_or(java_home);
            let java_home = java_home.file_name()?;
            let java_home = java_home.to_string_lossy();

            return Some(write!(f, " ☕️ {}", java_home.color(CYAN)));
        }
    }

    None
}

fn branch_name(repo: &Repository, f: &mut fmt::Formatter) -> fmt::Result {
    branch_name_by_head(repo, f).unwrap_or_else(|| branch_name_by_describe(repo, f))
}

fn branch_name_by_head(repo: &Repository, f: &mut fmt::Formatter) -> Option<fmt::Result> {
    let mut head = unwrap!(repo.head());
    let kind = head.kind()?;
    if kind == ReferenceType::Symbolic {
        head = unwrap!(head.resolve());
    }
    let write_result = if head.is_branch() {
        let head = head.shorthand()?;
        write!(f, " {}", head.color(VIOLET))
    } else {
        let id = head.target()?;
        let id = format!("{}", id);
        write!(f, " {}", id[..=9].color(PURPLE))
    };
    Some(write_result)
}

fn branch_name_by_describe(repo: &Repository, f: &mut fmt::Formatter) -> fmt::Result {
    let mut describe_opts = DescribeOptions::default();
    describe_opts.describe_all().max_candidates_tags(0);
    let describe = repo.describe(&describe_opts).and_then(|d| d.format(None));
    match describe {
        Ok(s) => write!(f, "{}", s.color(VIOLET)),
        Err(_) => write!(f, "{}", "(unknown)".color(VIOLET)),
    }
}

fn repo_state(repo: &Repository, f: &mut fmt::Formatter) -> fmt::Result {
    use git2::RepositoryState::*;

    let state = match repo.state() {
        Clean => return Ok(()),
        Merge => "merge",
        Revert | RevertSequence => "revert",
        CherryPick | CherryPickSequence => "cherry-pick",
        Bisect => "bisect",
        Rebase | RebaseInteractive | RebaseMerge => "rebase",
        ApplyMailbox | ApplyMailboxOrRebase => "am",
    };
    write!(f, " {}", state.color(PURPLE))
}

fn sys_info(smc: &mut Smc, f: &mut fmt::Formatter) -> fmt::Result {
    let system = System::new();
    let num_cpus = map_err!(smc.cpu_core_temps()).count() as f64;
    let load = system.get_load_average().one;
    let load = {
        let factor = load / num_cpus;
        if factor > 4.0 {
            "😰"
        } else if factor > 3.0 {
            "😥"
        } else if factor > 2.0 {
            "😓"
        } else if factor > 1.0 {
            "😅"
        } else {
            ""
        }
    };

    write!(f, " {}", load.dim())
}
