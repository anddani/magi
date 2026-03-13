Output showcase.gif

Set Shell zsh
Set FontSize 16
Set Width 1200
Set Height 800

# Start magi in the demo repo
Type "cd assets/demo/repo && magi"
Enter

Sleep 700ms

# Navigate down through sections
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Expand staged section diff
Tab
Sleep 700ms

# Move down and expand unstaged section diff
Type "j"
Sleep 300ms
Tab
Sleep 700ms

# Open help popup
Type "?"
Sleep 700ms

# Close help
Type "q"
Sleep 300ms

# Stage the unstaged file with visual mode
Type "j"
Sleep 300ms
Type "V"
Sleep 300ms
Type "jj"
Sleep 300ms
Type "s"
Sleep 700ms

# Open commit popup
Type "c"
Sleep 500ms

# Confirm commit
Type "c"
Sleep 700ms

# Type commit message and save
Type "IAdd showcase changes"
Sleep 300ms
Escape
Sleep 300ms
Type ":wq"
Enter
Sleep 700ms

# Open log view
Type "l"
Sleep 300ms
Type "a"
Sleep 700ms

# Navigate commits
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Preview a commit
Enter
Sleep 700ms

# Exit preview
Type "q"
Sleep 300ms

# Quit log
Type "q"
Sleep 300ms

# Quit magi
Type "q"
Sleep 500ms
