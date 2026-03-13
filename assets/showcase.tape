Output assets/showcase.gif

Set Shell zsh
Set FontSize 16
Set Width 1200
Set Height 800

# Start magi in the demo repo
Type "magi assets/demo/repo"
Enter

Sleep 700ms

# Navigate down through sections
Type "jjjjjj"
Sleep 300ms

# Expand staged section diff
Tab
Sleep 700ms

# Search inside the buffer for the change
Type "/format"
Enter
Sleep 300ms

# Stage the unstaged file with visual mode
Type "V"
Sleep 300ms
Type "j"
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
Enter
Sleep 1200ms

# Exit preview
Type "q"
Sleep 300ms

# Quit log
Type "q"
Sleep 300ms

# Quit magi
Type "q"
Sleep 500ms
