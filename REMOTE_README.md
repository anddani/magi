
# Upstream vs pushRemote

The distinction exists to support different collaboration workflows:

  When They're the Same (Simple Workflow)

  Most developers only need upstream. You clone a repo, create a branch, and both pull and push from the same remote (origin):

  origin/main ←→ local/main
       ↑↓
     (both pull and push go here)

  In this case, you don't need to set pushRemote at all.

  When They Differ (Triangular Workflow)

  pushRemote becomes useful when contributing to projects you don't have write access to:

  upstream/main → local/main → origin/main
       ↓                            ↑
     (pull from here)         (push to here - your fork)

  Example: Contributing to an open-source project:
  1. Fork github.com/project/repo to github.com/you/repo
  2. Clone your fork as origin
  3. Add the original as upstream
  4. Set upstream tracking to upstream/main (where you pull updates from)
  5. Set pushRemote to origin (your fork, where you push feature branches)

  This lets you:
  - git pull to get latest changes from the canonical repo
  - git push to send your work to your fork for PRs

  Other Use Cases

  - Backup remotes: Push to a secondary remote automatically
  - CI/staging remotes: Push to a deployment remote while tracking production
  - Team forks: Pull from team lead's branch, push to your own

  The Key Trade-off
  ┌───────────────────────┬────────────────────────────────────┬───────────────────────────────────┐
  │        Setting        │            Flexibility             │            Complexity             │
  ├───────────────────────┼────────────────────────────────────┼───────────────────────────────────┤
  │ Upstream only         │ Simple, one remote does everything │ Can't separate pull/push targets  │
  ├───────────────────────┼────────────────────────────────────┼───────────────────────────────────┤
  │ Upstream + pushRemote │ Full triangular workflow support   │ Two things to configure and track │
  └───────────────────────┴────────────────────────────────────┴───────────────────────────────────┘
  If you're working on repos where you have direct push access, you likely never need pushRemote. It's specifically designed for the fork-and-PR contribution model.
