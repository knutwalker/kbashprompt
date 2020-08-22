use super::{ModuleInfo, ModuleOut};
use colorful::Color;
use git2::{DescribeOptions, ReferenceType, Repository};

const VIOLET: Color = Color::SlateBlue3a; // 61
const PURPLE: Color = Color::DeepPink4c; // 125

#[path = "java_version.rs"]
mod java_version;
#[path = "rust_version.rs"]
mod rust_version;

pub(super) fn try_with_each<E>(mut fun: impl FnMut(ModuleInfo) -> Result<(), E>) -> Result<(), E> {
    if let Ok(repo) = Repository::open_from_env() {
        try_with_each_repo_type(&repo, &mut fun)?;
        fun(branch_name(&repo))?;
        if let Some(info) = repo_state(&repo) {
            fun(info)?;
        }
    }
    Ok(())
}

fn try_with_each_repo_type<E>(
    repo: &Repository,
    mut fun: impl FnMut(ModuleInfo) -> Result<(), E>,
) -> Result<(), E> {
    if let Some(workdir) = repo.workdir() {
        if let Some(v) = rust_version::get(workdir) {
            fun(v)?
        }
        if let Some(v) = java_version::get(workdir) {
            fun(v)?
        }
    }
    Ok(())
}

fn branch_name(repo: &Repository) -> ModuleInfo {
    branch_name_by_head(repo).unwrap_or_else(|| branch_name_by_describe(repo))
}

fn branch_name_by_head(repo: &Repository) -> ModuleOut {
    let mut head = repo.head().ok()?;
    let kind = head.kind()?;
    if kind == ReferenceType::Symbolic {
        head = head.resolve().ok()?;
    }
    let branch = if head.is_branch() {
        let head = head.shorthand()?;
        ModuleInfo::of().text(head).color(VIOLET)
    } else {
        let id = head.target()?;
        let id = format!("{:.9}", id);
        ModuleInfo::of().info(id).color(PURPLE)
    };
    Some(branch)
}

fn branch_name_by_describe(repo: &Repository) -> ModuleInfo {
    let info = ModuleInfo::of().color(VIOLET);
    let mut describe_opts = DescribeOptions::default();
    describe_opts.describe_all().max_candidates_tags(0);
    let describe = repo.describe(&describe_opts).and_then(|d| d.format(None));
    match describe {
        Ok(s) => info.info(s),
        Err(_) => info.info("(unknown)"),
    }
}

fn repo_state(repo: &Repository) -> ModuleOut {
    use git2::RepositoryState::*;

    let state = match repo.state() {
        Clean => return None,
        Merge => "merge",
        Revert | RevertSequence => "revert",
        CherryPick | CherryPickSequence => "cherry-pick",
        Bisect => "bisect",
        Rebase | RebaseInteractive | RebaseMerge => "rebase",
        ApplyMailbox | ApplyMailboxOrRebase => "am",
    };
    ModuleInfo::of().info(state).some()
}
