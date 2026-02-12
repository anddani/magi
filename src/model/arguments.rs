use std::collections::HashSet;

pub enum Arguments {
    CommitArguments(HashSet<CommitArgument>),
    FetchArguments(HashSet<FetchArgument>),
    PushArguments(HashSet<PushArgument>),
    PullArguments(HashSet<PullArgument>),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Argument {
    Commit(CommitArgument),
    Fetch(FetchArgument),
    Push(PushArgument),
    Pull(PullArgument),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum CommitArgument {
    StageAll,
    AllowEmpty,
    Verbose,
    DisableHooks,
}

impl CommitArgument {
    pub fn all() -> Vec<CommitArgument> {
        vec![
            CommitArgument::StageAll,
            CommitArgument::AllowEmpty,
            CommitArgument::Verbose,
            CommitArgument::DisableHooks,
        ]
    }

    pub fn key(&self) -> char {
        match self {
            CommitArgument::StageAll => 'a',
            CommitArgument::AllowEmpty => 'e',
            CommitArgument::Verbose => 'v',
            CommitArgument::DisableHooks => 'n',
        }
    }

    pub fn from_key(key: char) -> Option<CommitArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }

    pub fn description(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "Stage all modified and deleted files",
            CommitArgument::AllowEmpty => "Allow empty commit",
            CommitArgument::Verbose => "Show diff of changes to be commited",
            CommitArgument::DisableHooks => "Disable hooks",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "--all",
            CommitArgument::AllowEmpty => "--allow-empty",
            CommitArgument::Verbose => "--verbose",
            CommitArgument::DisableHooks => "--no-verify",
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
    pub fn all() -> Vec<FetchArgument> {
        vec![
            FetchArgument::Prune,
            FetchArgument::Tags,
            FetchArgument::Force,
        ]
    }

    pub fn key(&self) -> char {
        match self {
            FetchArgument::Prune => 'p',
            FetchArgument::Tags => 't',
            FetchArgument::Force => 'F',
        }
    }

    pub fn from_key(key: char) -> Option<FetchArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }

    pub fn description(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "Prune deleted branches",
            FetchArgument::Tags => "Fetch all tags",
            FetchArgument::Force => "Force",
        }
    }

    pub fn flag(&self) -> &'static str {
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
    /// Returns all possible push arguments
    pub fn all() -> Vec<PushArgument> {
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

    pub fn key(&self) -> char {
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

    pub fn from_key(key: char) -> Option<PushArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }

    pub fn description(&self) -> &'static str {
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

    pub fn flag(&self) -> &'static str {
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
}

impl PullArgument {
    pub fn all() -> Vec<PullArgument> {
        vec![
            PullArgument::FfOnly,
            PullArgument::Rebase,
            PullArgument::Autostash,
        ]
    }

    pub fn key(&self) -> char {
        match self {
            PullArgument::FfOnly => 'f',
            PullArgument::Rebase => 'r',
            PullArgument::Autostash => 'a',
        }
    }

    pub fn from_key(key: char) -> Option<PullArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }

    pub fn description(&self) -> &'static str {
        match self {
            PullArgument::FfOnly => "Fast-forward only",
            PullArgument::Rebase => "Rebase local commits",
            PullArgument::Autostash => "Autostash",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            PullArgument::FfOnly => "--ff-only",
            PullArgument::Rebase => "--rebase",
            PullArgument::Autostash => "--autostash",
        }
    }
}
