# VHS demo: before/after hero GIF
# Run: bash docs/demo-setup.sh && cd /tmp/lowfat-demo && vhs ../docs/demo.tape

Output "demo.gif"

Require lowfat

Set Shell "bash"
Set FontSize 15
Set Width 800
Set Height 500
Set WindowBar Colorful
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 80ms

Env PS1 "$ "
Env GIT_PAGER "less"
Env LOWFAT_LEVEL "ultra"

# --- git diff ---

# Before: raw git diff — scroll through with less
Type "echo '--- without lowfat ---'"
Enter
Sleep 1s
Type "git diff"
Enter
Sleep 2s

# Scroll down page by page
Space
Sleep 1200ms
Space
Sleep 1200ms
Space
Sleep 1200ms
Space
Sleep 1200ms
Space
Sleep 1200ms
Space
Sleep 1200ms
Space
Sleep 1200ms

# Quit less
Type "q"
Sleep 500ms

# After: ultra level
Ctrl+L
Sleep 300ms
Type "echo '--- with lowfat ---'"
Enter
Sleep 1s
Type "lowfat git diff"
Enter
Sleep 3s

# --- git log ---

# Before: raw git log
Ctrl+L
Sleep 300ms
Type "echo '--- without lowfat ---'"
Enter
Sleep 1s
Type "git log"
Enter
Sleep 3s

# Quit less
Type "q"
Sleep 500ms

# After: ultra level
Ctrl+L
Sleep 300ms
Type "echo '--- with lowfat ---'"
Enter
Sleep 1s
Type "lowfat git log"
Enter
Sleep 3s

# --- token savings ---
Ctrl+L
Sleep 300ms
Type "lowfat gain"
Enter
Sleep 4s
