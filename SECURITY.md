# Security policy

mdgrid reads and writes your notes, so problems that could change or expose files matter to us.

## Reporting a vulnerability

Please report it privately with GitHub's **Report a vulnerability** button on the repository's Security tab (private vulnerability reporting). Do not open a public issue.

Include what you found, how to reproduce it, and what an attacker could do with it. We will acknowledge the report, keep you informed, and credit you in the release notes unless you prefer otherwise.

## What counts

- mdgrid writing outside the edited value, outside the opened folder, or to a file it should have left read-only.
- Terminal escape sequences or control characters in note contents reaching the terminal.
- A note or `.base` file that makes mdgrid run a command or read files it should not.

Crashes on malformed input are bugs, not vulnerabilities, unless they lose or corrupt data; please open a normal issue for them.

## Supported versions

Only the latest release receives fixes.
