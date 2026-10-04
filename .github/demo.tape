# SPDX-FileCopyrightText: 2026 Sebastien Rousseau <sebastian.rousseau@gmail.com>
# SPDX-License-Identifier: Apache-2.0 OR MIT

Output .github/demo.gif

Set Shell "bash"
Set FontSize 15
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 35ms

Hide
Type "export PS1='$ ' && clear"
Enter
Show

Type "./target/release/oxml --version"
Enter
Sleep 2s

Type "./target/release/oxml --help"
Enter
Sleep 3s

Type "echo '<books><book><title>Dune</title></book></books>' | ./target/release/oxml query '//title' -t"
Enter
Sleep 3s

Type "echo '<books><book><title>Dune</title></book></books>' | ./target/release/oxml stats"
Enter
Sleep 3s
