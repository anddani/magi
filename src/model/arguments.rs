use std::{collections::HashSet, hash::Hash};

pub enum Arguments {
    CommitArguments(HashSet<CommitArgument>),
    FetchArguments(HashSet<FetchArgument>),
    PushArguments(HashSet<PushArgument>),
    PullArguments(HashSet<PullArgument>),
    StashArguments(HashSet<StashArgument>),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Argument {
    Commit(CommitArgument),
    Fetch(FetchArgument),
    Push(PushArgument),
    Pull(PullArgument),
    Stash(StashArgument),
}

pub trait PopupArgument: Sized + Eq + Hash {
    fn all() -> Vec<Self>;
    fn key(&self) -> char;
    fn description(&self) -> &'static str;
    fn flag(&self) -> &'static str;
}

impl Arguments {
    pub fn commit(&self) -> Option<&HashSet<CommitArgument>> {
        if let Arguments::CommitArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn fetch(&self) -> Option<&HashSet<FetchArgument>> {
        if let Arguments::FetchArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn push(&self) -> Option<&HashSet<PushArgument>> {
        if let Arguments::PushArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn pull(&self) -> Option<&HashSet<PullArgument>> {
        if let Arguments::PullArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn stash(&self) -> Option<&HashSet<StashArgument>> {
        if let Arguments::StashArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn commit_mut(&mut self) -> Option<&mut HashSet<CommitArgument>> {
        if let Arguments::CommitArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn fetch_mut(&mut self) -> Option<&mut HashSet<FetchArgument>> {
        if let Arguments::FetchArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn push_mut(&mut self) -> Option<&mut HashSet<PushArgument>> {
        if let Arguments::PushArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn pull_mut(&mut self) -> Option<&mut HashSet<PullArgument>> {
        if let Arguments::PullArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn stash_mut(&mut self) -> Option<&mut HashSet<StashArgument>> {
        if let Arguments::StashArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum CommitArgument {
    StageAll,
    AllowEmpty,
    Verbose,
    DisableHooks,
    ResetAuthor,
}

impl CommitArgument {
    pub fn from_key(key: char) -> Option<CommitArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for CommitArgument {
    fn all() -> Vec<CommitArgument> {
        vec![
            CommitArgument::StageAll,
            CommitArgument::AllowEmpty,
            CommitArgument::Verbose,
            CommitArgument::DisableHooks,
            CommitArgument::ResetAuthor,
        ]
    }

    fn key(&self) -> char {
        match self {
            CommitArgument::StageAll => 'a',
            CommitArgument::AllowEmpty => 'e',
            CommitArgument::Verbose => 'v',
            CommitArgument::DisableHooks => 'n',
            CommitArgument::ResetAuthor => 'R',
        }
    }

    fn description(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "Stage all modified and deleted files",
            CommitArgument::AllowEmpty => "Allow empty commit",
            CommitArgument::Verbose => "Show diff of changes to be commited",
            CommitArgument::DisableHooks => "Disable hooks",
            CommitArgument::ResetAuthor => "Claim authorship and reset author date",
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "--all",
            CommitArgument::AllowEmpty => "--allow-empty",
            CommitArgument::Verbose => "--verbose",
            CommitArgument::DisableHooks => "--no-verify",
            CommitArgument::ResetAuthor => "--reset-author",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum FetchArgument {
    Prune,
    Tags,
    Force,
}

impl FetchArgument {
    pub fn from_key(key: char) -> Option<FetchArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for FetchArgument {
    fn all() -> Vec<FetchArgument> {
        vec![
            FetchArgument::Prune,
            FetchArgument::Tags,
            FetchArgument::Force,
        ]
    }

    fn key(&self) -> char {
        match self {
            FetchArgument::Prune => 'p',
            FetchArgument::Tags => 't',
            FetchArgument::Force => 'F',
        }
    }

    fn description(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "Prune deleted branches",
            FetchArgument::Tags => "Fetch all tags",
            FetchArgument::Force => "Force",
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "--prune",
            FetchArgument::Tags => "--tags",
            FetchArgument::Force => "--force",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum PushArgument {
    ForceWithLease,
    Force,
    DisableHooks,
    DryRun,
    SetUpstream,
    IncludeAllTags,
    IncludeRelatedAnnotatedTags,
}

impl PushArgument {
    pub fn from_key(key: char) -> Option<PushArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for PushArgument {
    fn all() -> Vec<PushArgument> {
        vec![
            PushArgument::ForceWithLease,
            PushArgument::Force,
            PushArgument::DisableHooks,
            PushArgument::DryRun,
            PushArgument::SetUpstream,
            PushArgument::IncludeAllTags,
            PushArgument::IncludeRelatedAnnotatedTags,
        ]
    }

    fn key(&self) -> char {
        match self {
            PushArgument::ForceWithLease => 'f',
            PushArgument::Force => 'F',
            PushArgument::DisableHooks => 'h',
            PushArgument::DryRun => 'n',
            PushArgument::SetUpstream => 'u',
            PushArgument::IncludeAllTags => 'T',
            PushArgument::IncludeRelatedAnnotatedTags => 't',
        }
    }

    fn description(&self) -> &'static str {
        match self {
            PushArgument::ForceWithLease => "Force with lease",
            PushArgument::Force => "Force",
            PushArgument::DisableHooks => "Disable hooks",
            PushArgument::DryRun => "Dry run",
            PushArgument::SetUpstream => "Set upstream",
            PushArgument::IncludeAllTags => "Include all tags",
            PushArgument::IncludeRelatedAnnotatedTags => "Include related annotated tags",
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            PushArgument::ForceWithLease => "--force-with-lease",
            PushArgument::Force => "--force",
            PushArgument::DisableHooks => "--no-verify",
            PushArgument::DryRun => "--dry-run",
            PushArgument::SetUpstream => "--set-upstream",
            PushArgument::IncludeAllTags => "--tags",
            PushArgument::IncludeRelatedAnnotatedTags => "--follow-tags",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum PullArgument {
    FfOnly,
    Rebase,
    Autostash,
    Force,
}

impl PullArgument {
    pub fn from_key(key: char) -> Option<PullArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for PullArgument {
    fn all() -> Vec<PullArgument> {
        vec![
            PullArgument::FfOnly,
            PullArgument::Rebase,
            PullArgument::Autostash,
            PullArgument::Force,
        ]
    }

    fn key(&self) -> char {
        match self {
            PullArgument::FfOnly => 'f',
            PullArgument::Rebase => 'r',
            PullArgument::Autostash => 'a',
            PullArgument::Force => 'F',
        }
    }

    fn description(&self) -> &'static str {
        match self {
            PullArgument::FfOnly => "Fast-forward only",
            PullArgument::Rebase => "Rebase local commits",
            PullArgument::Autostash => "Autostash",
            PullArgument::Force => "Force",
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            PullArgument::FfOnly => "--ff-only",
            PullArgument::Rebase => "--rebase",
            PullArgument::Autostash => "--autostash",
            PullArgument::Force => "--force",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum StashArgument {
    IncludeUntracked,
    All,
}

impl StashArgument {
    pub fn from_key(key: char) -> Option<StashArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for StashArgument {
    fn all() -> Vec<StashArgument> {
        vec![StashArgument::IncludeUntracked, StashArgument::All]
    }

    fn key(&self) -> char {
        match self {
            StashArgument::IncludeUntracked => 'u',
            StashArgument::All => 'a',
        }
    }

    fn description(&self) -> &'static str {
        match self {
            StashArgument::IncludeUntracked => "Also save untracked files",
            StashArgument::All => "Also save untracked and ignored files",
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            StashArgument::IncludeUntracked => "--include-untracked",
            StashArgument::All => "--all",
        }
    }
}
