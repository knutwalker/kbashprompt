use chrono::Local;
use colorful::{Color, Colorful, Style};
use git2::{DescribeOptions, DiffDelta, ReferenceType, Repository, StatusOptions};
use home::home_dir;
use std::{
    env::{self, args},
    fmt::{self, Display},
    io::{stdout, Write},
    ops::{BitAnd, BitOrAssign},
    path::{Path, PathBuf},
    process::Command,
};

const _WHITE: Color = Color::White; // 15
const BLUE: Color = Color::DodgerBlue1; // 33
const CYAN: Color = Color::LightSeaGreen; // 37
const VIOLET: Color = Color::SlateBlue3a; // 61
const GREEN: Color = Color::Chartreuse4; // 64
const RED: Color = Color::Red3a; // 124
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

struct Ps1;

impl Display for Ps1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        current_time(f)?;
        current_dir(f)?;
        git_prompt(f)?;
        writeln!(f, "")?;
        write!(f, "∵ ")
    }
}

struct Ps2;

impl Display for Ps2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "→ ".color(YELLOW))
    }
}

fn current_time(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        f,
        "{}",
        Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .style(Style::Dim)
    )
}

fn current_dir(f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

fn git_prompt(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Ok(repo) = Repository::open_from_env() {
        repo_type(&repo, f).unwrap_or(Ok(()))?;
        branch_name(&repo, f)?;
        repo_changes(&repo, f).unwrap_or(Ok(()))?;
        repo_state(&repo, f)?;
    }
    Ok(())
}

fn branch_name(repo: &Repository, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    branch_name_by_head(repo, f).unwrap_or_else(|| branch_name_by_describe(repo, f))
}

fn branch_name_by_head(repo: &Repository, f: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
    let mut head = repo.head().ok()?;
    let kind = head.kind()?;
    if kind == ReferenceType::Symbolic {
        head = head.resolve().ok()?;
    }
    let name = head.shorthand()?;
    Some(write!(f, " {}", name.color(VIOLET)))
}

fn branch_name_by_describe(repo: &Repository, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut describe_opts = DescribeOptions::default();
    describe_opts.describe_all().max_candidates_tags(0);
    let describe = repo.describe(&describe_opts).and_then(|d| d.format(None));
    match describe {
        Ok(s) => write!(f, "{}", s.color(VIOLET)),
        Err(_) => write!(f, "{}", "(unknown)".color(VIOLET)),
    }
}

fn repo_state(repo: &Repository, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
enum GitStatus {
    Added = 1,
    Deleted = 2,
    Modified = 4,
    New = 8,
    Conflict = 16,
}

impl BitOrAssign<GitStatus> for u8 {
    #[inline]
    fn bitor_assign(&mut self, rhs: GitStatus) {
        *self |= rhs as u8;
    }
}

impl BitAnd<GitStatus> for u8 {
    type Output = bool;

    #[inline]
    fn bitand(self, rhs: GitStatus) -> Self::Output {
        (self & rhs as u8) == (rhs as u8)
    }
}

fn repo_changes(repo: &Repository, f: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
    let mut status_options = StatusOptions::new();
    let status_options = status_options
        .include_untracked(true)
        .include_ignored(false)
        .include_unmodified(false)
        .include_unreadable(false);

    let mut changes = 0;

    for state in repo.statuses(Some(status_options)).ok()?.iter() {
        file_change(state.index_to_workdir(), &mut changes);
        file_change(state.head_to_index(), &mut changes);
    }

    if changes == 0 {
        return None;
    }

    let mut buf = String::with_capacity(changes.count_ones() as usize + 2);

    buf.push('[');
    if changes & GitStatus::Conflict {
        buf.push('!');
    }
    if changes & GitStatus::Added {
        buf.push('A');
    }
    if changes & GitStatus::Deleted {
        buf.push('D');
    }
    if changes & GitStatus::Modified {
        buf.push('M');
    }
    if changes & GitStatus::New {
        buf.push('?');
    }
    buf.push(']');

    let color = if changes & GitStatus::Conflict {
        RED
    } else {
        BLUE
    };

    Some(write!(f, " {}", buf.color(color)))
}

fn file_change(state: Option<DiffDelta>, flag: &mut u8) {
    use git2::Delta::*;

    if let Some(state) = state {
        match state.status() {
            Added => *flag |= GitStatus::Added,
            Deleted => *flag |= GitStatus::Deleted,
            Modified | Renamed | Typechange => *flag |= GitStatus::Modified,
            Untracked => *flag |= GitStatus::New,
            Conflicted => *flag |= GitStatus::Conflict,
            Unmodified | Copied | Ignored | Unreadable => {}
        }
    }
}

fn repo_type(repo: &Repository, f: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
    let wd = repo.workdir()?;
    if let Some(Err(e)) = rust_version(wd, f) {
        return Some(Err(e));
    }
    if let Some(Err(e)) = java_version(wd, f) {
        return Some(Err(e));
    }

    Some(Ok(()))
}

fn rust_version(wd: impl AsRef<Path>, f: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
    let cargo_toml = wd.as_ref().join("Cargo.toml");
    let cargo_toml = cargo_toml.metadata().ok()?;
    if !cargo_toml.is_file() {
        return None;
    }

    let rustc_version = Command::new("rustc").arg("--version").output().ok()?;
    if !rustc_version.status.success() {
        return None;
    }

    let rustc_version = String::from_utf8(rustc_version.stdout).ok()?;
    let rustc_version = rustc_version.split_whitespace().nth(1)?;

    Some(write!(f, " 🦀 {}", rustc_version.color(ORANGE)))
}

fn java_version(wd: impl AsRef<Path>, f: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
    let has_file = move |file: &str| -> Option<()> {
        let build_file = wd.as_ref().join(file);
        build_file
            .metadata()
            .ok()
            .filter(|f| f.is_file())
            .map(|_| ())
    };

    for file in [
        "build.gradle",
        "build.sbt",
        "pom.xml",
        "build.gradle.kts",
        "build.xml",
    ]
    .iter()
    {
        if has_file(file).is_some() {
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
