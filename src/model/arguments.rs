use std::collections::HashSet;

pub enum Arguments {
    FetchArguments(HashSet<FetchArgument>),
    PushArguments(HashSet<PushArgument>),
    PullArguments(HashSet<PullArgument>),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Argument {
    Fetch(FetchArgument),
    Push(PushArgument),
    Pull(PullArgument),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum FetchArgument {
    Prune,
}

impl FetchArgument {
    pub fn all() -> Vec<FetchArgument> {
        vec![FetchArgument::Prune]
    }

    pub fn key(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "p",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "Prune deleted branches",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            FetchArgument::Prune => "--prune",
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
