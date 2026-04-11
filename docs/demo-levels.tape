# VHS demo: intensity levels (lite → full → ultra)
# Run: bash docs/demo-setup.sh && cd /tmp/lowfat-demo && vhs ../docs/demo-levels.tape

Output "demo-levels.gif"

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

Type "LOWFAT_LEVEL=lite lowfat git diff"
Enter
Sleep 3s

Type "LOWFAT_LEVEL=full lowfat git diff"
Enter
Sleep 3s

Type "LOWFAT_LEVEL=ultra lowfat git diff"
Enter
Sleep 3s
