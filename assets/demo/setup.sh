#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$SCRIPT_DIR/repo"

echo "Setting up demo repo at $REPO_DIR..."

# Clean slate
rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR/src"

cd "$REPO_DIR"
git init
git config user.name "Demo User"
git config user.email "demo@example.com"

# Commit 1: initial project skeleton
cat > README.md << 'EOF'
# my-project

A small Rust utility.

## Usage

```
cargo run
```
EOF

mkdir -p src
cat > src/main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

cat > Cargo.toml << 'EOF'
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"
EOF

git add .
git commit -m "Initial commit"

# Commit 2: add a helper function
cat > src/main.rs << 'EOF'
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    println!("{}", greet("world"));
}
EOF

git add src/main.rs
git commit -m "Add greet helper function"

# Commit 3: update README with contributing section
cat >> README.md << 'EOF'

## Contributing

Pull requests welcome!
EOF

git add README.md
git commit -m "Add contributing section to README"

# --- Working tree state for the demo ---

# Staged change: update README title line
cat > README.md << 'EOF'
# my-project — now with magi!

A small Rust utility.

## Usage

```
cargo run
```

## Contributing

Pull requests welcome!
EOF

git add README.md

# Unstaged change: edit src/main.rs
cat > src/main.rs << 'EOF'
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to my-project.", name)
}

fn main() {
    let name = "world";
    println!("{}", greet(name));
}
EOF

# Untracked file
cat > TODO.md << 'EOF'
# TODO

- [ ] Add CLI argument parsing
- [ ] Write tests
- [ ] Publish to crates.io
EOF

echo ""
echo "Done! Demo repo state:"
git status
