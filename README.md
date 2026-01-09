# 🧙 Magi

A terminal-based Git client.

![Status: In development](https://img.shields.io/badge/Status-In%20Development-yellow)
![Language: Rust](https://img.shields.io/badge/Language-Rust-orange)
![License: MIT](https://img.shields.io/badge/License-MIT-blue)

---

## Features and goals

Magi is inspired by [Magit](https://magit.vc/), the legendary Emacs Git interface. The goal of this project is to create an as faithful Magit experience as possible for the terminal, removing the need for Emacs.

- **Keyboard-Centric Interface**: Efficient Git operations without leaving the terminal
- **Faithful emulation of Magit**: Features are as closely matched with Magit
- **Vi(m) bindings first class citizen**: Vim editor style navigation
- **No Emacs Required**: Brings Magit-like power to any developer workflow

## Roadmap

- [x] Repository status view (HEAD, push ref, tags)
- [x] Untracked files display
- [x] Staged/unstaged changes with inline diffs
- [x] Stage and unstage files
- [x] Expand/Collapse sections
- [ ] Keyboard navigation
    - [x] Move up/down
    - [ ] Scroll viewport
    - [ ] Visual select
- [x] User dismissable dialog
- [x] Toast
- [x] Configurable color themes
- [ ] Commands
    - [ ] Apply
    - [ ] Branch
    - [ ] Bisect
    - [ ] Commit
        - [x] Commit
        - [ ] Amend
        - [ ] Reword
        - [ ] Extend
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
    - [ ] Pull
    - [ ] Help
    - [ ] Jump to section
    - [ ] Log
    - [ ] Log (change)
    - [ ] Merge
    - [ ] Remote
    - [ ] Submodule
    - [ ] Subtree
    - [ ] Push
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
    - [ ] Stage
    - [ ] Unstage
    - [ ] Stage all
    - [ ] Unstage all
- [ ] Push and pull operations
- [ ] Branch management (create, checkout, delete)
- [ ] Stash support
- [ ] Interactive rebasing
- [ ] Log view with commit history

## How does Magi differ from Gitu?

[Gitu](https://github.com/altsem/gitu) is a great Magit-inspired terminal Git client that might fit your needs. However, the design of Gitu is more of a mix of [Magit](https://magit.vc/) and [Lazygit](https://github.com/jesseduffield/lazygit) (another great Git TUI). Gitu gets rid of the "editor buffer" experience in favor of a more streamlined, and limited, navigation and interaction experience. This is not necessarily a bad thing, but it does differ from the experience we are used to in Magit.

Here are some list of things that Magi **can** do and Gitu **cannot**:

- **Search through buffer**: By pressing '/', you can search through the information to quickly find specific lines in a diff.
- **Visual select**: Entering visual select using 'V' will allow you to stage a range of lines in a hunk, select a few files to stage, or a few stashes to drop.
- **Faithful keybindings**: Magi will preserve the default keybindings in Magit+Evil in order to make onboarding easier.


## License

[MIT](./LICENSE)
