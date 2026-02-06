use std::collections::HashSet;

pub enum Arguments {
    PushArguments(HashSet<PushArgument>),
    PullArguments(HashSet<PullArgument>),
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
            PushArgument::ForceWithLease => "force with lease",
            PushArgument::Force => "force",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            PushArgument::ForceWithLease => "--force-with-lease",
            PushArgument::Force => "--force",
        }
    }
}

pub enum PullArgument {
    Force,
}

impl Into<String> for PushArgument {
    fn into(self) -> String {
        String::from(match self {
            PushArgument::ForceWithLease => "--force-with-lease",
            PushArgument::Force => "--force",
        })
    }
}
