use std::{collections::HashSet, hash::Hash};

use crate::i18n;

/// Default maximum number of commits shown in the log view, used when the `-n`
/// argument is left at its default (mirrors git's own lack of a limit while
/// keeping the view responsive, like Magit's default).
pub const DEFAULT_LOG_LIMIT: usize = 256;

pub enum Arguments {
    CommitArguments(HashSet<CommitArgument>),
    FetchArguments(HashSet<FetchArgument>),
    PushArguments(HashSet<PushArgument>),
    PullArguments(HashSet<PullArgument>),
    StashArguments(HashSet<StashArgument>),
    StashPushArguments(HashSet<StashPushArgument>),
    RevertArguments(HashSet<RevertArgument>),
    LogArguments {
        args: HashSet<LogArgument>,
        /// Maximum number of commits to show, set via the `-n` argument (`-n<N>`).
        /// Defaults to [`DEFAULT_LOG_LIMIT`].
        limit: usize,
        /// Author pattern to limit the log to, set via the `-A` argument
        /// (`--author=<pattern>`). `None` means no author filter.
        author: Option<String>,
        /// Pattern to search commit messages for, set via the `-F` argument
        /// (`--grep=<pattern>`). `None` means no message filter.
        grep: Option<String>,
        /// Pattern to search added/removed changes for, set via the `-G`
        /// argument (`-G<pattern>`). `None` means no change search.
        search_changes: Option<String>,
        /// Pattern to search changed occurrences for, set via the `-S`
        /// argument (`-S<pattern>`). `None` means no occurrence search.
        search_occurrences: Option<String>,
        /// Line range to trace the evolution of, set via the `-L` argument
        /// (`-L<start>,<end>:<file>`). `None` means no line trace.
        trace_lines: Option<String>,
        /// Pathspec(s) to limit the log to, set via the `--` argument.
        /// `None` means no file filter.
        files: Option<String>,
    },
    TagArguments {
        args: HashSet<TagArgument>,
        /// Key to sign the tag with, set via the `-u` argument (`--local-user=`)
        local_user: Option<String>,
    },
    RebaseArguments {
        args: HashSet<RebaseArgument>,
        /// GPG signing key set via the `-S` argument (`--gpg-sign=`)
        gpg_sign: Option<String>,
    },
    MergeArguments {
        args: HashSet<MergeArgument>,
        /// Merge strategy set via the `-s` argument (`--strategy=`)
        strategy: Option<String>,
        /// Diff algorithm set via the `-A` argument (`-Xdiff-algorithm=`)
        diff_algorithm: Option<String>,
        /// GPG signing key set via the `-S` argument (`--gpg-sign=`)
        gpg_sign: Option<String>,
    },
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Argument {
    Commit(CommitArgument),
    Fetch(FetchArgument),
    Push(PushArgument),
    Pull(PullArgument),
    Stash(StashArgument),
    StashPush(StashPushArgument),
    Revert(RevertArgument),
    Log(LogArgument),
    Tag(TagArgument),
    Rebase(RebaseArgument),
    Merge(MergeArgument),
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

    pub fn revert(&self) -> Option<&HashSet<RevertArgument>> {
        if let Arguments::RevertArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    /// Log arguments with the commit limit at its default ([`DEFAULT_LOG_LIMIT`])
    /// and no author filter
    pub fn log_args(args: HashSet<LogArgument>) -> Arguments {
        Arguments::LogArguments {
            args,
            limit: DEFAULT_LOG_LIMIT,
            author: None,
            grep: None,
            search_changes: None,
            search_occurrences: None,
            trace_lines: None,
            files: None,
        }
    }

    pub fn log(&self) -> Option<&HashSet<LogArgument>> {
        if let Arguments::LogArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn log_limit(&self) -> Option<usize> {
        if let Arguments::LogArguments { limit, .. } = self {
            Some(*limit)
        } else {
            None
        }
    }

    pub fn log_limit_mut(&mut self) -> Option<&mut usize> {
        if let Arguments::LogArguments { limit, .. } = self {
            Some(limit)
        } else {
            None
        }
    }

    pub fn log_author(&self) -> Option<&str> {
        if let Arguments::LogArguments { author, .. } = self {
            author.as_deref()
        } else {
            None
        }
    }

    pub fn log_author_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::LogArguments { author, .. } = self {
            Some(author)
        } else {
            None
        }
    }

    pub fn log_grep(&self) -> Option<&str> {
        if let Arguments::LogArguments { grep, .. } = self {
            grep.as_deref()
        } else {
            None
        }
    }

    pub fn log_grep_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::LogArguments { grep, .. } = self {
            Some(grep)
        } else {
            None
        }
    }

    pub fn log_search_changes(&self) -> Option<&str> {
        if let Arguments::LogArguments { search_changes, .. } = self {
            search_changes.as_deref()
        } else {
            None
        }
    }

    pub fn log_search_changes_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::LogArguments { search_changes, .. } = self {
            Some(search_changes)
        } else {
            None
        }
    }

    pub fn log_search_occurrences(&self) -> Option<&str> {
        if let Arguments::LogArguments {
            search_occurrences, ..
        } = self
        {
            search_occurrences.as_deref()
        } else {
            None
        }
    }

    pub fn log_search_occurrences_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::LogArguments {
            search_occurrences, ..
        } = self
        {
            Some(search_occurrences)
        } else {
            None
        }
    }

    pub fn log_trace_lines(&self) -> Option<&str> {
        if let Arguments::LogArguments { trace_lines, .. } = self {
            trace_lines.as_deref()
        } else {
            None
        }
    }

    pub fn log_trace_lines_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::LogArguments { trace_lines, .. } = self {
            Some(trace_lines)
        } else {
            None
        }
    }

    pub fn log_files(&self) -> Option<&str> {
        if let Arguments::LogArguments { files, .. } = self {
            files.as_deref()
        } else {
            None
        }
    }

    pub fn log_files_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::LogArguments { files, .. } = self {
            Some(files)
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

    pub fn stash_push(&self) -> Option<&HashSet<StashPushArgument>> {
        if let Arguments::StashPushArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn stash_push_mut(&mut self) -> Option<&mut HashSet<StashPushArgument>> {
        if let Arguments::StashPushArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn revert_mut(&mut self) -> Option<&mut HashSet<RevertArgument>> {
        if let Arguments::RevertArguments(args) = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn log_mut(&mut self) -> Option<&mut HashSet<LogArgument>> {
        if let Arguments::LogArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    /// Tag arguments with no `--local-user=` override set
    pub fn tag_args(args: HashSet<TagArgument>) -> Arguments {
        Arguments::TagArguments {
            args,
            local_user: None,
        }
    }

    pub fn tag(&self) -> Option<&HashSet<TagArgument>> {
        if let Arguments::TagArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn tag_mut(&mut self) -> Option<&mut HashSet<TagArgument>> {
        if let Arguments::TagArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn tag_local_user(&self) -> Option<&str> {
        if let Arguments::TagArguments { local_user, .. } = self {
            local_user.as_deref()
        } else {
            None
        }
    }

    pub fn tag_local_user_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::TagArguments { local_user, .. } = self {
            Some(local_user)
        } else {
            None
        }
    }

    /// Rebase arguments with no `--gpg-sign=` override set
    pub fn rebase_args(args: HashSet<RebaseArgument>) -> Arguments {
        Arguments::RebaseArguments {
            args,
            gpg_sign: None,
        }
    }

    pub fn rebase(&self) -> Option<&HashSet<RebaseArgument>> {
        if let Arguments::RebaseArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn rebase_mut(&mut self) -> Option<&mut HashSet<RebaseArgument>> {
        if let Arguments::RebaseArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn rebase_gpg_sign(&self) -> Option<&str> {
        if let Arguments::RebaseArguments { gpg_sign, .. } = self {
            gpg_sign.as_deref()
        } else {
            None
        }
    }

    pub fn rebase_gpg_sign_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::RebaseArguments { gpg_sign, .. } = self {
            Some(gpg_sign)
        } else {
            None
        }
    }

    /// Merge arguments with no `--strategy=`, `-Xdiff-algorithm=`, or
    /// `--gpg-sign=` override set
    pub fn merge_args(args: HashSet<MergeArgument>) -> Arguments {
        Arguments::MergeArguments {
            args,
            strategy: None,
            diff_algorithm: None,
            gpg_sign: None,
        }
    }

    pub fn merge(&self) -> Option<&HashSet<MergeArgument>> {
        if let Arguments::MergeArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn merge_mut(&mut self) -> Option<&mut HashSet<MergeArgument>> {
        if let Arguments::MergeArguments { args, .. } = self {
            Some(args)
        } else {
            None
        }
    }

    pub fn merge_strategy(&self) -> Option<&str> {
        if let Arguments::MergeArguments { strategy, .. } = self {
            strategy.as_deref()
        } else {
            None
        }
    }

    pub fn merge_strategy_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::MergeArguments { strategy, .. } = self {
            Some(strategy)
        } else {
            None
        }
    }

    pub fn merge_diff_algorithm(&self) -> Option<&str> {
        if let Arguments::MergeArguments { diff_algorithm, .. } = self {
            diff_algorithm.as_deref()
        } else {
            None
        }
    }

    pub fn merge_diff_algorithm_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::MergeArguments { diff_algorithm, .. } = self {
            Some(diff_algorithm)
        } else {
            None
        }
    }

    pub fn merge_gpg_sign(&self) -> Option<&str> {
        if let Arguments::MergeArguments { gpg_sign, .. } = self {
            gpg_sign.as_deref()
        } else {
            None
        }
    }

    pub fn merge_gpg_sign_mut(&mut self) -> Option<&mut Option<String>> {
        if let Arguments::MergeArguments { gpg_sign, .. } = self {
            Some(gpg_sign)
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
    GpgSign,
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
            CommitArgument::GpgSign,
        ]
    }

    fn key(&self) -> char {
        match self {
            CommitArgument::StageAll => 'a',
            CommitArgument::AllowEmpty => 'e',
            CommitArgument::Verbose => 'v',
            CommitArgument::DisableHooks => 'n',
            CommitArgument::ResetAuthor => 'R',
            CommitArgument::GpgSign => 'S',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            CommitArgument::StageAll => t.arg_commit_stage_all,
            CommitArgument::AllowEmpty => t.arg_commit_allow_empty,
            CommitArgument::Verbose => t.arg_commit_verbose,
            CommitArgument::DisableHooks => t.arg_commit_disable_hooks,
            CommitArgument::ResetAuthor => t.arg_commit_reset_author,
            CommitArgument::GpgSign => t.arg_commit_gpg_sign,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            CommitArgument::StageAll => "--all",
            CommitArgument::AllowEmpty => "--allow-empty",
            CommitArgument::Verbose => "--verbose",
            CommitArgument::DisableHooks => "--no-verify",
            CommitArgument::ResetAuthor => "--reset-author",
            CommitArgument::GpgSign => "--gpg-sign",
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
        let t = i18n::t();
        match self {
            FetchArgument::Prune => t.arg_fetch_prune,
            FetchArgument::Tags => t.arg_fetch_tags,
            FetchArgument::Force => t.arg_fetch_force,
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
        let t = i18n::t();
        match self {
            PushArgument::ForceWithLease => t.arg_push_force_with_lease,
            PushArgument::Force => t.arg_push_force,
            PushArgument::DisableHooks => t.arg_push_disable_hooks,
            PushArgument::DryRun => t.arg_push_dry_run,
            PushArgument::SetUpstream => t.arg_push_set_upstream,
            PushArgument::IncludeAllTags => t.arg_push_include_all_tags,
            PushArgument::IncludeRelatedAnnotatedTags => t.arg_push_include_related_tags,
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
        let t = i18n::t();
        match self {
            PullArgument::FfOnly => t.arg_pull_ff_only,
            PullArgument::Rebase => t.arg_pull_rebase,
            PullArgument::Autostash => t.arg_pull_autostash,
            PullArgument::Force => t.arg_pull_force,
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
        let t = i18n::t();
        match self {
            StashArgument::IncludeUntracked => t.arg_stash_include_untracked,
            StashArgument::All => t.arg_stash_all,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            StashArgument::IncludeUntracked => "--include-untracked",
            StashArgument::All => "--all",
        }
    }
}

/// Arguments for the "Stash push" popup (`git stash push`). Unlike the main
/// stash popup, this one also exposes the keep-index switches, matching
/// magit's dedicated `magit-stash-push` transient.
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum StashPushArgument {
    IncludeUntracked,
    All,
    KeepIndex,
    NoKeepIndex,
}

impl StashPushArgument {
    pub fn from_key(key: char) -> Option<StashPushArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for StashPushArgument {
    fn all() -> Vec<StashPushArgument> {
        vec![
            StashPushArgument::IncludeUntracked,
            StashPushArgument::All,
            StashPushArgument::KeepIndex,
            StashPushArgument::NoKeepIndex,
        ]
    }

    fn key(&self) -> char {
        match self {
            StashPushArgument::IncludeUntracked => 'u',
            StashPushArgument::All => 'a',
            StashPushArgument::KeepIndex => 'k',
            StashPushArgument::NoKeepIndex => 'K',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            StashPushArgument::IncludeUntracked => t.arg_stash_include_untracked,
            StashPushArgument::All => t.arg_stash_all,
            StashPushArgument::KeepIndex => t.arg_stash_keep_index,
            StashPushArgument::NoKeepIndex => t.arg_stash_no_keep_index,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            StashPushArgument::IncludeUntracked => "--include-untracked",
            StashPushArgument::All => "--all",
            StashPushArgument::KeepIndex => "--keep-index",
            StashPushArgument::NoKeepIndex => "--no-keep-index",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum RevertArgument {
    Edit,
    NoEdit,
}

impl RevertArgument {
    pub fn from_key(key: char) -> Option<RevertArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum LogArgument {
    Graph,
    Color,
    Decorate,
    ShowHeader,
    Patch,
    Stat,
    ShowSignature,
    SimplifyByDecoration,
    Follow,
}

impl LogArgument {
    pub fn from_key(key: char) -> Option<LogArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for LogArgument {
    /// `all()` holds only the `-` toggles rendered in the Formatting section.
    /// ShowSignature is excluded: it uses the `=` prefix (magit's `=S`), so it
    /// is rendered with `prefixed_argument_line` and toggled in equals-arg mode.
    /// SimplifyByDecoration and Follow are excluded too: they live in the
    /// "History simplification" section and are toggled via explicit `-D`/`-f`
    /// key handlers.
    fn all() -> Vec<LogArgument> {
        vec![
            LogArgument::Graph,
            LogArgument::Color,
            LogArgument::Decorate,
            LogArgument::ShowHeader,
            LogArgument::Patch,
            LogArgument::Stat,
        ]
    }

    fn key(&self) -> char {
        match self {
            LogArgument::Graph => 'g',
            LogArgument::Color => 'c',
            LogArgument::Decorate => 'd',
            LogArgument::ShowHeader => 'h',
            LogArgument::Patch => 'p',
            LogArgument::Stat => 's',
            LogArgument::ShowSignature => 'S',
            LogArgument::SimplifyByDecoration => 'D',
            LogArgument::Follow => 'f',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            LogArgument::Graph => t.arg_log_graph,
            LogArgument::Color => t.arg_log_color,
            LogArgument::Decorate => t.arg_log_decorate,
            LogArgument::ShowHeader => t.arg_log_show_header,
            LogArgument::Patch => t.arg_log_patch,
            LogArgument::Stat => t.arg_log_stat,
            LogArgument::ShowSignature => t.arg_log_show_signature,
            LogArgument::SimplifyByDecoration => t.arg_log_simplify_by_decoration,
            LogArgument::Follow => t.arg_log_follow,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            LogArgument::Graph => "--graph",
            LogArgument::Color => "--color",
            LogArgument::Decorate => "--decorate",
            LogArgument::ShowHeader => "++header",
            LogArgument::Patch => "--patch",
            LogArgument::Stat => "--stat",
            LogArgument::ShowSignature => "--show-signature",
            LogArgument::SimplifyByDecoration => "--simplify-by-decoration",
            LogArgument::Follow => "--follow",
        }
    }
}

impl PopupArgument for RevertArgument {
    fn all() -> Vec<RevertArgument> {
        vec![RevertArgument::Edit, RevertArgument::NoEdit]
    }

    fn key(&self) -> char {
        match self {
            RevertArgument::Edit => 'e',
            RevertArgument::NoEdit => 'E',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            RevertArgument::Edit => t.arg_revert_edit,
            RevertArgument::NoEdit => t.arg_revert_no_edit,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            RevertArgument::Edit => "--edit",
            RevertArgument::NoEdit => "--no-edit",
        }
    }
}

/// Mode for `--rebase-merges=` (magit's `magit-rebase-merges-select-mode`)
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum RebaseMergesMode {
    NoRebaseCousins,
    RebaseCousins,
}

impl RebaseMergesMode {
    pub fn all() -> Vec<RebaseMergesMode> {
        vec![
            RebaseMergesMode::NoRebaseCousins,
            RebaseMergesMode::RebaseCousins,
        ]
    }

    pub fn from_value(value: &str) -> Option<RebaseMergesMode> {
        Self::all().into_iter().find(|mode| mode.value() == value)
    }

    pub fn value(&self) -> &'static str {
        match self {
            RebaseMergesMode::NoRebaseCousins => "no-rebase-cousins",
            RebaseMergesMode::RebaseCousins => "rebase-cousins",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum RebaseArgument {
    KeepEmpty,
    RebaseMerges(RebaseMergesMode),
    UpdateRefs,
    CommitterDateIsAuthorDate,
    IgnoreDate,
    Autosquash,
    Autostash,
    DisableHooks,
}

impl RebaseArgument {
    pub fn from_key(key: char) -> Option<RebaseArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for RebaseArgument {
    /// RebaseMerges is excluded: it carries a value, so it is rendered with
    /// `argument_value_line` and toggled via `Message::ToggleRebaseMerges`.
    fn all() -> Vec<RebaseArgument> {
        vec![
            RebaseArgument::KeepEmpty,
            RebaseArgument::UpdateRefs,
            RebaseArgument::CommitterDateIsAuthorDate,
            RebaseArgument::IgnoreDate,
            RebaseArgument::Autosquash,
            RebaseArgument::Autostash,
            RebaseArgument::DisableHooks,
        ]
    }

    fn key(&self) -> char {
        match self {
            RebaseArgument::KeepEmpty => 'k',
            RebaseArgument::RebaseMerges(_) => 'r',
            RebaseArgument::UpdateRefs => 'u',
            RebaseArgument::CommitterDateIsAuthorDate => 'd',
            RebaseArgument::IgnoreDate => 't',
            RebaseArgument::Autosquash => 'a',
            RebaseArgument::Autostash => 'A',
            RebaseArgument::DisableHooks => 'h',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            RebaseArgument::KeepEmpty => t.arg_rebase_keep_empty,
            RebaseArgument::RebaseMerges(_) => t.arg_rebase_rebase_merges,
            RebaseArgument::UpdateRefs => t.arg_rebase_update_refs,
            RebaseArgument::CommitterDateIsAuthorDate => t.arg_rebase_committer_date_is_author_date,
            RebaseArgument::IgnoreDate => t.arg_rebase_ignore_date,
            RebaseArgument::Autosquash => t.arg_rebase_autosquash,
            RebaseArgument::Autostash => t.arg_rebase_autostash,
            RebaseArgument::DisableHooks => t.arg_rebase_disable_hooks,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            RebaseArgument::KeepEmpty => "--keep-empty",
            RebaseArgument::RebaseMerges(RebaseMergesMode::NoRebaseCousins) => {
                "--rebase-merges=no-rebase-cousins"
            }
            RebaseArgument::RebaseMerges(RebaseMergesMode::RebaseCousins) => {
                "--rebase-merges=rebase-cousins"
            }
            RebaseArgument::UpdateRefs => "--update-refs",
            RebaseArgument::CommitterDateIsAuthorDate => "--committer-date-is-author-date",
            RebaseArgument::IgnoreDate => "--ignore-date",
            RebaseArgument::Autosquash => "--autosquash",
            RebaseArgument::Autostash => "--autostash",
            RebaseArgument::DisableHooks => "--no-verify",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum MergeArgument {
    FfOnly,
    NoFf,
    IgnoreSpaceChange,
    IgnoreAllSpace,
}

impl MergeArgument {
    pub fn from_key(key: char) -> Option<MergeArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for MergeArgument {
    fn all() -> Vec<MergeArgument> {
        vec![
            MergeArgument::FfOnly,
            MergeArgument::NoFf,
            MergeArgument::IgnoreSpaceChange,
            MergeArgument::IgnoreAllSpace,
        ]
    }

    fn key(&self) -> char {
        match self {
            MergeArgument::FfOnly => 'f',
            MergeArgument::NoFf => 'n',
            MergeArgument::IgnoreSpaceChange => 'b',
            MergeArgument::IgnoreAllSpace => 'w',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            MergeArgument::FfOnly => t.arg_merge_ff_only,
            MergeArgument::NoFf => t.arg_merge_no_ff,
            MergeArgument::IgnoreSpaceChange => t.arg_merge_ignore_space_change,
            MergeArgument::IgnoreAllSpace => t.arg_merge_ignore_all_space,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            MergeArgument::FfOnly => "--ff-only",
            MergeArgument::NoFf => "--no-ff",
            MergeArgument::IgnoreSpaceChange => "-Xignore-space-change",
            MergeArgument::IgnoreAllSpace => "-Xignore-all-space",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum TagArgument {
    Force,
    Edit,
    Annotate,
    Sign,
}

impl TagArgument {
    pub fn from_key(key: char) -> Option<TagArgument> {
        Self::all().into_iter().find(|arg| arg.key() == key)
    }
}

impl PopupArgument for TagArgument {
    fn all() -> Vec<TagArgument> {
        vec![
            TagArgument::Force,
            TagArgument::Edit,
            TagArgument::Annotate,
            TagArgument::Sign,
        ]
    }

    fn key(&self) -> char {
        match self {
            TagArgument::Force => 'f',
            TagArgument::Edit => 'e',
            TagArgument::Annotate => 'a',
            TagArgument::Sign => 's',
        }
    }

    fn description(&self) -> &'static str {
        let t = i18n::t();
        match self {
            TagArgument::Force => t.arg_tag_force,
            TagArgument::Edit => t.arg_tag_edit,
            TagArgument::Annotate => t.arg_tag_annotate,
            TagArgument::Sign => t.arg_tag_sign,
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            TagArgument::Force => "--force",
            TagArgument::Edit => "--edit",
            TagArgument::Annotate => "--annotate",
            TagArgument::Sign => "--sign",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_argument_gpg_sign_key_and_flag() {
        assert_eq!(CommitArgument::GpgSign.key(), 'S');
        assert_eq!(CommitArgument::GpgSign.flag(), "--gpg-sign");
    }

    #[test]
    fn test_commit_argument_from_key() {
        assert_eq!(CommitArgument::from_key('S'), Some(CommitArgument::GpgSign));
        assert_eq!(CommitArgument::from_key('x'), None);
    }

    #[test]
    fn test_commit_argument_all_contains_gpg_sign() {
        assert!(CommitArgument::all().contains(&CommitArgument::GpgSign));
    }

    #[test]
    fn test_revert_argument_edit_listed_above_no_edit() {
        assert_eq!(
            RevertArgument::all(),
            vec![RevertArgument::Edit, RevertArgument::NoEdit]
        );
    }

    #[test]
    fn test_revert_argument_keys_and_flags() {
        assert_eq!(RevertArgument::Edit.key(), 'e');
        assert_eq!(RevertArgument::Edit.flag(), "--edit");
        assert_eq!(RevertArgument::NoEdit.key(), 'E');
        assert_eq!(RevertArgument::NoEdit.flag(), "--no-edit");
    }

    #[test]
    fn test_revert_argument_from_key() {
        assert_eq!(RevertArgument::from_key('e'), Some(RevertArgument::Edit));
        assert_eq!(RevertArgument::from_key('E'), Some(RevertArgument::NoEdit));
        assert_eq!(RevertArgument::from_key('x'), None);
    }

    #[test]
    fn test_tag_argument_key_and_flag() {
        assert_eq!(TagArgument::Force.key(), 'f');
        assert_eq!(TagArgument::Force.flag(), "--force");
        assert_eq!(TagArgument::Edit.key(), 'e');
        assert_eq!(TagArgument::Edit.flag(), "--edit");
        assert_eq!(TagArgument::Annotate.key(), 'a');
        assert_eq!(TagArgument::Annotate.flag(), "--annotate");
        assert_eq!(TagArgument::Sign.key(), 's');
        assert_eq!(TagArgument::Sign.flag(), "--sign");
    }

    #[test]
    fn test_tag_argument_order_matches_magit() {
        assert_eq!(
            TagArgument::all(),
            vec![
                TagArgument::Force,
                TagArgument::Edit,
                TagArgument::Annotate,
                TagArgument::Sign,
            ]
        );
    }

    #[test]
    fn test_rebase_argument_key_and_flag() {
        assert_eq!(RebaseArgument::KeepEmpty.key(), 'k');
        assert_eq!(RebaseArgument::KeepEmpty.flag(), "--keep-empty");
    }

    #[test]
    fn test_rebase_argument_from_key() {
        assert_eq!(
            RebaseArgument::from_key('k'),
            Some(RebaseArgument::KeepEmpty)
        );
        assert_eq!(
            RebaseArgument::from_key('u'),
            Some(RebaseArgument::UpdateRefs)
        );
        assert_eq!(
            RebaseArgument::from_key('d'),
            Some(RebaseArgument::CommitterDateIsAuthorDate)
        );
        assert_eq!(
            RebaseArgument::from_key('t'),
            Some(RebaseArgument::IgnoreDate)
        );
        assert_eq!(
            RebaseArgument::from_key('a'),
            Some(RebaseArgument::Autosquash)
        );
        assert_eq!(RebaseArgument::from_key('x'), None);
    }

    #[test]
    fn test_rebase_argument_committer_date_is_author_date_key_and_flag() {
        assert_eq!(RebaseArgument::CommitterDateIsAuthorDate.key(), 'd');
        assert_eq!(
            RebaseArgument::CommitterDateIsAuthorDate.flag(),
            "--committer-date-is-author-date"
        );
    }

    #[test]
    fn test_rebase_argument_update_refs_key_and_flag() {
        assert_eq!(RebaseArgument::UpdateRefs.key(), 'u');
        assert_eq!(RebaseArgument::UpdateRefs.flag(), "--update-refs");
    }

    #[test]
    fn test_rebase_argument_all_order() {
        assert_eq!(
            RebaseArgument::all(),
            vec![
                RebaseArgument::KeepEmpty,
                RebaseArgument::UpdateRefs,
                RebaseArgument::CommitterDateIsAuthorDate,
                RebaseArgument::IgnoreDate,
                RebaseArgument::Autosquash,
                RebaseArgument::Autostash,
                RebaseArgument::DisableHooks,
            ]
        );
    }

    #[test]
    fn test_rebase_argument_autosquash_key_and_flag() {
        assert_eq!(RebaseArgument::Autosquash.key(), 'a');
        assert_eq!(RebaseArgument::Autosquash.flag(), "--autosquash");
    }

    #[test]
    fn test_rebase_argument_autostash_key_and_flag() {
        assert_eq!(RebaseArgument::Autostash.key(), 'A');
        assert_eq!(RebaseArgument::Autostash.flag(), "--autostash");
        assert_eq!(
            RebaseArgument::from_key('A'),
            Some(RebaseArgument::Autostash)
        );
    }

    #[test]
    fn test_rebase_argument_disable_hooks_key_and_flag() {
        assert_eq!(RebaseArgument::DisableHooks.key(), 'h');
        assert_eq!(RebaseArgument::DisableHooks.flag(), "--no-verify");
        assert_eq!(
            RebaseArgument::from_key('h'),
            Some(RebaseArgument::DisableHooks)
        );
    }

    #[test]
    fn test_rebase_args_has_no_gpg_sign() {
        let arguments = Arguments::rebase_args([RebaseArgument::Autostash].into_iter().collect());
        assert_eq!(arguments.rebase_gpg_sign(), None);
        assert!(
            arguments
                .rebase()
                .unwrap()
                .contains(&RebaseArgument::Autostash)
        );
    }

    #[test]
    fn test_rebase_gpg_sign_mut_sets_and_clears() {
        let mut arguments = Arguments::rebase_args(HashSet::new());
        *arguments.rebase_gpg_sign_mut().unwrap() = Some("ABCD1234".to_string());
        assert_eq!(arguments.rebase_gpg_sign(), Some("ABCD1234"));

        *arguments.rebase_gpg_sign_mut().unwrap() = None;
        assert_eq!(arguments.rebase_gpg_sign(), None);
    }

    #[test]
    fn test_rebase_gpg_sign_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.rebase_gpg_sign(), None);
    }

    #[test]
    fn test_rebase_merges_key_and_flags() {
        let no_cousins = RebaseArgument::RebaseMerges(RebaseMergesMode::NoRebaseCousins);
        let cousins = RebaseArgument::RebaseMerges(RebaseMergesMode::RebaseCousins);
        assert_eq!(no_cousins.key(), 'r');
        assert_eq!(cousins.key(), 'r');
        assert_eq!(no_cousins.flag(), "--rebase-merges=no-rebase-cousins");
        assert_eq!(cousins.flag(), "--rebase-merges=rebase-cousins");
    }

    #[test]
    fn test_rebase_merges_mode_from_value() {
        assert_eq!(
            RebaseMergesMode::from_value("no-rebase-cousins"),
            Some(RebaseMergesMode::NoRebaseCousins)
        );
        assert_eq!(
            RebaseMergesMode::from_value("rebase-cousins"),
            Some(RebaseMergesMode::RebaseCousins)
        );
        assert_eq!(RebaseMergesMode::from_value("bogus"), None);
    }

    #[test]
    fn test_rebase_argument_ignore_date_key_and_flag() {
        assert_eq!(RebaseArgument::IgnoreDate.key(), 't');
        assert_eq!(RebaseArgument::IgnoreDate.flag(), "--ignore-date");
    }

    #[test]
    fn test_rebase_merges_not_toggled_via_from_key() {
        // 'r' is handled by Message::ToggleRebaseMerges, not the generic toggle
        assert_eq!(RebaseArgument::from_key('r'), None);
    }

    #[test]
    fn test_log_argument_decorate_key_and_flag() {
        assert_eq!(LogArgument::Decorate.key(), 'd');
        assert_eq!(LogArgument::Decorate.flag(), "--decorate");
    }

    #[test]
    fn test_log_argument_from_key() {
        assert_eq!(LogArgument::from_key('g'), Some(LogArgument::Graph));
        assert_eq!(LogArgument::from_key('c'), Some(LogArgument::Color));
        assert_eq!(LogArgument::from_key('d'), Some(LogArgument::Decorate));
        assert_eq!(LogArgument::from_key('h'), Some(LogArgument::ShowHeader));
        assert_eq!(LogArgument::from_key('s'), Some(LogArgument::Stat));
        assert_eq!(LogArgument::from_key('x'), None);
    }

    #[test]
    fn test_log_argument_show_header_key_and_flag() {
        assert_eq!(LogArgument::ShowHeader.key(), 'h');
        assert_eq!(LogArgument::ShowHeader.flag(), "++header");
    }

    #[test]
    fn test_log_argument_stat_key_and_flag() {
        assert_eq!(LogArgument::Stat.key(), 's');
        assert_eq!(LogArgument::Stat.flag(), "--stat");
    }

    #[test]
    fn test_log_argument_show_signature_key_and_flag() {
        assert_eq!(LogArgument::ShowSignature.key(), 'S');
        assert_eq!(LogArgument::ShowSignature.flag(), "--show-signature");
    }

    #[test]
    fn test_log_argument_show_signature_not_toggled_via_from_key() {
        // 'S' is handled in equals-arg mode (`=S`), not the generic `-` toggle
        assert_eq!(LogArgument::from_key('S'), None);
    }

    #[test]
    fn test_log_argument_simplify_by_decoration_key_and_flag() {
        assert_eq!(LogArgument::SimplifyByDecoration.key(), 'D');
        assert_eq!(
            LogArgument::SimplifyByDecoration.flag(),
            "--simplify-by-decoration"
        );
    }

    #[test]
    fn test_log_argument_simplify_by_decoration_not_toggled_via_from_key() {
        // 'D' is handled by an explicit key binding (its own section), not the
        // generic `-` toggle driven by `all()`
        assert_eq!(LogArgument::from_key('D'), None);
    }

    #[test]
    fn test_log_argument_follow_key_and_flag() {
        assert_eq!(LogArgument::Follow.key(), 'f');
        assert_eq!(LogArgument::Follow.flag(), "--follow");
    }

    #[test]
    fn test_log_argument_follow_not_toggled_via_from_key() {
        // 'f' is handled by an explicit key binding (its own section), not the
        // generic `-` toggle driven by `all()`
        assert_eq!(LogArgument::from_key('f'), None);
    }

    #[test]
    fn test_log_argument_order_matches_magit() {
        assert_eq!(
            LogArgument::all(),
            vec![
                LogArgument::Graph,
                LogArgument::Color,
                LogArgument::Decorate,
                LogArgument::ShowHeader,
                LogArgument::Patch,
                LogArgument::Stat,
            ]
        );
    }

    #[test]
    fn test_merge_argument_key_and_flag() {
        assert_eq!(MergeArgument::FfOnly.key(), 'f');
        assert_eq!(MergeArgument::FfOnly.flag(), "--ff-only");
        assert_eq!(MergeArgument::NoFf.key(), 'n');
        assert_eq!(MergeArgument::NoFf.flag(), "--no-ff");
        assert_eq!(MergeArgument::IgnoreSpaceChange.key(), 'b');
        assert_eq!(
            MergeArgument::IgnoreSpaceChange.flag(),
            "-Xignore-space-change"
        );
        assert_eq!(MergeArgument::IgnoreAllSpace.key(), 'w');
        assert_eq!(MergeArgument::IgnoreAllSpace.flag(), "-Xignore-all-space");
    }

    #[test]
    fn test_merge_argument_from_key() {
        assert_eq!(MergeArgument::from_key('f'), Some(MergeArgument::FfOnly));
        assert_eq!(MergeArgument::from_key('n'), Some(MergeArgument::NoFf));
        assert_eq!(
            MergeArgument::from_key('b'),
            Some(MergeArgument::IgnoreSpaceChange)
        );
        assert_eq!(
            MergeArgument::from_key('w'),
            Some(MergeArgument::IgnoreAllSpace)
        );
        assert_eq!(MergeArgument::from_key('x'), None);
    }

    #[test]
    fn test_merge_argument_all_contains_all_variants() {
        assert!(MergeArgument::all().contains(&MergeArgument::FfOnly));
        assert!(MergeArgument::all().contains(&MergeArgument::NoFf));
        assert!(MergeArgument::all().contains(&MergeArgument::IgnoreSpaceChange));
        assert!(MergeArgument::all().contains(&MergeArgument::IgnoreAllSpace));
    }

    #[test]
    fn test_merge_args_has_no_strategy() {
        let arguments = Arguments::merge_args([MergeArgument::NoFf].into_iter().collect());
        assert_eq!(arguments.merge_strategy(), None);
        assert!(arguments.merge().unwrap().contains(&MergeArgument::NoFf));
    }

    #[test]
    fn test_merge_strategy_mut_sets_and_clears() {
        let mut arguments = Arguments::merge_args(HashSet::new());
        *arguments.merge_strategy_mut().unwrap() = Some("ours".to_string());
        assert_eq!(arguments.merge_strategy(), Some("ours"));

        *arguments.merge_strategy_mut().unwrap() = None;
        assert_eq!(arguments.merge_strategy(), None);
    }

    #[test]
    fn test_merge_strategy_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.merge_strategy(), None);
    }

    #[test]
    fn test_merge_args_has_no_diff_algorithm() {
        let arguments = Arguments::merge_args([MergeArgument::NoFf].into_iter().collect());
        assert_eq!(arguments.merge_diff_algorithm(), None);
        assert!(arguments.merge().unwrap().contains(&MergeArgument::NoFf));
    }

    #[test]
    fn test_merge_diff_algorithm_mut_sets_and_clears() {
        let mut arguments = Arguments::merge_args(HashSet::new());
        *arguments.merge_diff_algorithm_mut().unwrap() = Some("histogram".to_string());
        assert_eq!(arguments.merge_diff_algorithm(), Some("histogram"));

        *arguments.merge_diff_algorithm_mut().unwrap() = None;
        assert_eq!(arguments.merge_diff_algorithm(), None);
    }

    #[test]
    fn test_merge_diff_algorithm_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.merge_diff_algorithm(), None);
    }

    #[test]
    fn test_merge_strategy_and_diff_algorithm_are_independent() {
        let mut arguments = Arguments::merge_args(HashSet::new());
        *arguments.merge_strategy_mut().unwrap() = Some("ours".to_string());
        *arguments.merge_diff_algorithm_mut().unwrap() = Some("patience".to_string());
        assert_eq!(arguments.merge_strategy(), Some("ours"));
        assert_eq!(arguments.merge_diff_algorithm(), Some("patience"));
    }

    #[test]
    fn test_merge_args_has_no_gpg_sign() {
        let arguments = Arguments::merge_args([MergeArgument::NoFf].into_iter().collect());
        assert_eq!(arguments.merge_gpg_sign(), None);
        assert!(arguments.merge().unwrap().contains(&MergeArgument::NoFf));
    }

    #[test]
    fn test_merge_gpg_sign_mut_sets_and_clears() {
        let mut arguments = Arguments::merge_args(HashSet::new());
        *arguments.merge_gpg_sign_mut().unwrap() = Some("ABCD1234".to_string());
        assert_eq!(arguments.merge_gpg_sign(), Some("ABCD1234"));

        *arguments.merge_gpg_sign_mut().unwrap() = None;
        assert_eq!(arguments.merge_gpg_sign(), None);
    }

    #[test]
    fn test_merge_gpg_sign_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.merge_gpg_sign(), None);
    }

    #[test]
    fn test_tag_args_has_no_local_user() {
        let arguments = Arguments::tag_args([TagArgument::Force].into_iter().collect());
        assert_eq!(arguments.tag_local_user(), None);
        assert!(arguments.tag().unwrap().contains(&TagArgument::Force));
    }

    #[test]
    fn test_log_args_defaults_to_default_limit() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_limit(), Some(DEFAULT_LOG_LIMIT));
    }

    #[test]
    fn test_log_limit_mut_sets_value() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_limit_mut().unwrap() = 42;
        assert_eq!(arguments.log_limit(), Some(42));
    }

    #[test]
    fn test_log_limit_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_limit(), None);
    }

    #[test]
    fn test_log_args_defaults_to_no_author() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_author(), None);
    }

    #[test]
    fn test_log_author_mut_sets_and_clears() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_author_mut().unwrap() = Some("Ada".to_string());
        assert_eq!(arguments.log_author(), Some("Ada"));

        *arguments.log_author_mut().unwrap() = None;
        assert_eq!(arguments.log_author(), None);
    }

    #[test]
    fn test_log_author_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_author(), None);
    }

    #[test]
    fn test_log_args_defaults_to_no_grep() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_grep(), None);
    }

    #[test]
    fn test_log_grep_mut_sets_and_clears() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_grep_mut().unwrap() = Some("fix".to_string());
        assert_eq!(arguments.log_grep(), Some("fix"));

        *arguments.log_grep_mut().unwrap() = None;
        assert_eq!(arguments.log_grep(), None);
    }

    #[test]
    fn test_log_grep_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_grep(), None);
    }

    #[test]
    fn test_log_args_defaults_to_no_search_changes() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_search_changes(), None);
    }

    #[test]
    fn test_log_search_changes_mut_sets_and_clears() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_search_changes_mut().unwrap() = Some("TODO".to_string());
        assert_eq!(arguments.log_search_changes(), Some("TODO"));

        *arguments.log_search_changes_mut().unwrap() = None;
        assert_eq!(arguments.log_search_changes(), None);
    }

    #[test]
    fn test_log_search_changes_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_search_changes(), None);
    }

    #[test]
    fn test_log_args_defaults_to_no_search_occurrences() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_search_occurrences(), None);
    }

    #[test]
    fn test_log_search_occurrences_mut_sets_and_clears() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_search_occurrences_mut().unwrap() = Some("foo".to_string());
        assert_eq!(arguments.log_search_occurrences(), Some("foo"));

        *arguments.log_search_occurrences_mut().unwrap() = None;
        assert_eq!(arguments.log_search_occurrences(), None);
    }

    #[test]
    fn test_log_search_occurrences_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_search_occurrences(), None);
    }

    #[test]
    fn test_log_args_defaults_to_no_trace_lines() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_trace_lines(), None);
    }

    #[test]
    fn test_log_trace_lines_mut_sets_and_clears() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_trace_lines_mut().unwrap() = Some("1,10:src/main.rs".to_string());
        assert_eq!(arguments.log_trace_lines(), Some("1,10:src/main.rs"));

        *arguments.log_trace_lines_mut().unwrap() = None;
        assert_eq!(arguments.log_trace_lines(), None);
    }

    #[test]
    fn test_log_trace_lines_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_trace_lines(), None);
    }

    #[test]
    fn test_log_args_defaults_to_no_files() {
        let arguments = Arguments::log_args(HashSet::new());
        assert_eq!(arguments.log_files(), None);
    }

    #[test]
    fn test_log_files_mut_sets_and_clears() {
        let mut arguments = Arguments::log_args(HashSet::new());
        *arguments.log_files_mut().unwrap() = Some("src/main.rs".to_string());
        assert_eq!(arguments.log_files(), Some("src/main.rs"));

        *arguments.log_files_mut().unwrap() = None;
        assert_eq!(arguments.log_files(), None);
    }

    #[test]
    fn test_log_files_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.log_files(), None);
    }

    #[test]
    fn test_tag_local_user_mut_sets_and_clears() {
        let mut arguments = Arguments::tag_args(HashSet::new());
        *arguments.tag_local_user_mut().unwrap() = Some("ABCD1234".to_string());
        assert_eq!(arguments.tag_local_user(), Some("ABCD1234"));

        *arguments.tag_local_user_mut().unwrap() = None;
        assert_eq!(arguments.tag_local_user(), None);
    }

    #[test]
    fn test_tag_local_user_on_other_arguments_is_none() {
        let arguments = Arguments::CommitArguments(HashSet::new());
        assert_eq!(arguments.tag_local_user(), None);
    }

    #[test]
    fn test_tag_argument_from_key() {
        assert_eq!(TagArgument::from_key('f'), Some(TagArgument::Force));
        assert_eq!(TagArgument::from_key('e'), Some(TagArgument::Edit));
        assert_eq!(TagArgument::from_key('a'), Some(TagArgument::Annotate));
        assert_eq!(TagArgument::from_key('s'), Some(TagArgument::Sign));
        assert_eq!(TagArgument::from_key('x'), None);
    }
}
