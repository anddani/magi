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
}
impl CommitArgument {
    pub fn all() -> Vec<CommitArgument> {
        vec![CommitArgument::StageAll]
    }

    pub fn key(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "a",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "Stage all modified and deleted files",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "--all",
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

    pub fn key(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "p",
            FetchArgument::Tags => "t",
            FetchArgument::Force => "F",
        }
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
}

impl PushArgument {
    /// Returns all possible push arguments
    pub fn all() -> Vec<PushArgument> {
        vec![PushArgument::ForceWithLease, PushArgument::Force]
    }

    pub fn key(&self) -> &'static str {
        match self {
            PushArgument::ForceWithLease => "f",
            PushArgument::Force => "F",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PushArgument::ForceWithLease => "Force with lease",
            PushArgument::Force => "Force",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            PushArgument::ForceWithLease => "--force-with-lease",
            PushArgument::Force => "--force",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum PullArgument {
    Force,
}
