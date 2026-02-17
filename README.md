# 🧙 Magi {—*}
A terminal-based Git client.

![Status: In development](https://img.shields.io/badge/Status-In%20Development-yellow)
![Language: Rust](https://img.shields.io/badge/Language-Rust-orange)
[![Built With Ratatui](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=fff&labelColor=000&color=fff)](https://ratatui.rs)
![License: MIT](https://img.shields.io/badge/License-MIT-blue)


---

## Features and goals

Magi is inspired by [Magit](https://magit.vc/), the legendary Emacs Git interface. The goal of this project is to create an as faithful Magit experience as possible for the terminal, removing the need for Emacs.

- **Keyboard-Centric Interface**
- **Faithful emulation of Magit**
- **Vi(m) bindings first class citizen**
- **No Emacs Required**

## Motivation

There are many Git TUIs out there. Here are a couple:

- [Gitu](https://github.com/altsem/gitu)
- [Lazygit](https://github.com/jesseduffield/lazygit)
- [Gitui](https://github.com/gitui-org/gitui)

They are all great but what they lack is the "editor like" experience you get with [Magit](https://magit.vc/).
This project aims to allow [Magit](https://magit.vc/) users to use this application with low friction.
Here are a few features that the aforementioned applications lack:

- **Search through buffer**: By pressing '/', you can search through the information in the visible buffer
- **Visual select**: Entering visual select using 'V' will allow you to stage a range of lines in a hunk, select a few files to stage, or a few stashes to drop
- **Faithful keybindings**: Magi will preserve the default keybindings in Magit+Evil (and potentially Emacs bindings) in order to make onboarding easier
- **Legible commit graph**: Easy navigation and overview of the commit graph
- **Contextual commands**: All commands are available at any time, using the highlighted or selected line(s) as context to automatically figure out intent

## Roadmap

- [x] Repository status view (HEAD, push ref, tags)
- [x] Untracked files display
- [x] Staged/unstaged changes with inline diffs
- [x] Stage and unstage files
- [x] Expand/Collapse sections
- [x] Keyboard navigation
    - [x] Move up/down
    - [x] Scroll viewport
    - [x] Visual select
- [x] User dismissable popup
- [x] Toast
- [x] Configurable color themes
- [ ] Commands
    - [ ] Apply
    - [ ] Branch
        - [x] Checkout branch/revision
        - [x] Checkout local branch
        - [x] Checkout new branch
        - [ ] Checkout spin-off branch
        - [ ] Checkout new worktree
        - [x] Create new branch
        - [ ] Create new spin-out branch
        - [ ] Create new worktree
    - [ ] Bisect
    - [ ] Commit
        - [x] Commit
        - [x] Amend
        - [x] Reword
        - [x] Extend
        - [ ] Fixup
        - [ ] Squash
        - [ ] Alter
        - [ ] Augment
        - [ ] Revise
        - [ ] Instant fixup
        - [ ] Instant squash
    - [ ] Clone
    - [ ] Diff
    - [ ] Diff (change)
    - [ ] Fetch
        - [x] Fetch all remotes
        - [x] Prune deleted branches
        - [x] Fetch all tags
        - [x] Force
    - [ ] Pull
        - [x] Fast forward only
        - [x] Rebase local commits
        - [x] Autostash
        - [x] Force
    - [ ] Help
    - [ ] Jump to section
    - [ ] Log
        - [x] Local
        - [ ] Other
        - [ ] Related
        - [ ] Local branches
        - [ ] All branches
        - [x] All references
        - [ ] Current reflog
        - [ ] Other reflog
        - [ ] HEAD reflog
        - [ ] Shortlog
    - [ ] Log (change)
    - [ ] Merge
    - [ ] Remote
    - [ ] Submodule
    - [ ] Subtree
    - [ ] Push
        - [x] Force with lease
        - [x] Force
        - [x] Disable hooks
        - [x] Dry run
        - [x] Set upstream
        - [x] Include all tags
        - [x] Include related annotated tags
    - [ ] Rebase
    - [ ] Tag
    - [ ] Note
    - [ ] Revert
    - [ ] Apply patches
    - [ ] Format patches
    - [ ] Reset
    - [ ] Show refs
    - [ ] Stash
    - [ ] Worktree
- [ ] Applying changes
    - [ ] Apply
    - [ ] Reverse
    - [ ] Discard
    - [x] Stage
    - [ ] Unstage
    - [x] Stage all
    - [x] Unstage all


## License

[MIT](./LICENSE)
