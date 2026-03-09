<div align="center">
    <h1>Magi</h1>
    <p>A terminal-based Git client.</p>
    <img src="assets/monitor.jpg" alt="Banner">
</div>

---

<div align="center">
    <img alt="Status: In development" src="https://img.shields.io/badge/Status-In%20Development-yellow" />
    <img alt="Language: Rust" src="https://img.shields.io/badge/Language-Rust-orange" />
    <a href="https://ratatui.rs"><img alt="Built with Ratatui" src="https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=fff&labelColor=000&color=fff" /></a>
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue" />
</div>

## Features and goals

Magi is inspired by [Magit](https://magit.vc/), the legendary Emacs Git interface. The goal of this project is to create an as faithful Magit experience as possible for the terminal, removing the need for Emacs.

- **Keyboard-Centric Interface**
- **Faithful emulation of Magit**
- **Vi(m) bindings first class citizen**
- **No Emacs Required**

<img alt="Welcome to VHS" src="assets/magi.gif" />

## Installation

```
# Homebrew
brew tap anddani/homebrew-magi
brew install magi

# Nix
nix-env -iA nixpkgs.magi

# Arch Linux
pacman -S magi
    
```

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
    - [x] Branch
    - [ ] Bisect
    - [ ] Commit
        - [x] Commit
        - [x] Amend
        - [x] Reword
        - [x] Extend
        - [x] Fixup
        - [x] Squash
        - [x] Alter
        - [x] Augment
        - [ ] Revise
        - [ ] Instant fixup
        - [ ] Instant squash
    - [ ] Clone
    - [x] Fetch
    - [x] Pull
    - [x] Help
    - [ ] Log
        - [x] Local
        - [ ] Other
        - [ ] Related
        - [x] Local branches
        - [x] All branches
        - [x] All references
        - [ ] Current reflog
        - [ ] Other reflog
        - [ ] HEAD reflog
        - [ ] Shortlog
    - [ ] Merge
    - [ ] Remote
    - [ ] Submodule
    - [ ] Subtree
    - [x] Push
    - [ ] Rebase
    - [ ] Tag
    - [ ] Note
    - [ ] Revert
    - [ ] Apply patches
    - [ ] Format patches
    - [x] Reset
    - [ ] Show refs
    - [ ] Stash
    - [ ] Worktree
- [ ] Applying changes
    - [ ] Apply
    - [ ] Reverse
    - [x] Discard
    - [x] Stage
    - [x] Unstage
    - [x] Stage all
    - [x] Unstage all


## License

[MIT](./LICENSE)
