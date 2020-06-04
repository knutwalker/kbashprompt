use chrono::Local;
use colorful::{Color, Colorful, Style};
use git2::{DescribeOptions, ReferenceType, Repository};
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
    use git2::Delta::*;

    let mut changes = 0;
    for state in repo.statuses(None).ok()?.iter() {
        if let Some(state) = state.index_to_workdir() {
            match state.status() {
                Added => changes |= GitStatus::Added,
                Deleted => changes |= GitStatus::Deleted,
                Modified | Renamed | Typechange => changes |= GitStatus::Modified,
                Untracked => changes |= GitStatus::New,
                Conflicted => changes |= GitStatus::Conflict,
                Unmodified | Copied | Ignored | Unreadable => {}
            }
        }
    }

    if changes == 0 {
        return None;
    }

    let mut buf = String::with_capacity(changes.count_ones() as usize + 2);

    buf.push('[');
    if changes & GitStatus::Conflict {
        buf.push('!');
    }
    if changes & GitStatus::New {
        buf.push('?');
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
    buf.push(']');

    let color = if changes & GitStatus::Conflict {
        RED
    } else {
        BLUE
    };

    Some(write!(f, " {}", buf.color(color)))
}
