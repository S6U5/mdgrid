# Write-back safety

[日本語](safety.ja.md)

mdgrid edits your notes in place, so it is built around one promise: it changes only the values you edited (adding a key line or a frontmatter block when needed), and leaves every other byte of the file as it was. This page describes what mdgrid writes, which notes and values it refuses to write, what happens when a note changes on disk while you are editing it, and how a file is actually written. It describes the current behavior.

The examples of read-only notes below are checked by a test (`tests/test_safety_docs.rs`): each example is run through the frontmatter reader, and the test fails if it does not come out read-only for the stated reason, or if the reader gains a reason that this page does not list.

## What mdgrid changes

Edits are not written immediately. mdgrid keeps them as pending changes (marked in the table, and counted in the bottom bar), and writes them only when you save (`Ctrl+S` or `:w` by default). Before writing, it shows a diff for each file and asks you to confirm. Pending changes can be undone and redone, and if you quit with pending changes, mdgrid asks whether to save, discard, or go back.

When it writes, mdgrid changes only:

- **The value of the edited key.** It finds the byte range of that key's value in the frontmatter and replaces just that range. Other keys, key order, comments, blank lines, line endings (LF or CRLF), and the body stay byte-for-byte the same. After writing, only the edited value differs in a diff.
- **One new line for a missing key.** If the note has frontmatter but not the key, mdgrid adds a `key: value` line at the end of the frontmatter. A list value is added in the vertical form Obsidian uses (`key:` followed by `  - item` lines).
- **A new frontmatter block, for notes without one.** If the note has no frontmatter (its first line is not `---`), mdgrid adds `---`, `key: value`, `---` at the very top and leaves the body bytes as they were. If the note has empty frontmatter (just two `---` lines), mdgrid adds the key line between them. This can be turned off; see [`no_frontmatter`](#no_frontmatter) below.
- **A key's name, or a whole key, when you rename or delete a key across notes** (the column actions "Rename the key" and "Delete the key"). Renaming replaces only the bytes of the key name on its line; the value, its quoting and any comment stay. Deleting removes the key's line and the lines of its value (list items, multi-line text); blank lines and comments after it stay, since they usually belong to the next key.

New lines use the file's own line ending (LF for a file with no line break at all).

mdgrid writes only top-level keys of a note's frontmatter. It never edits a note's body (`e` opens the note in your editor instead), never rewrites a `.base` file, and does not replace any other existing file. The only files it creates are a new note (along with any missing folders above it), a new `.base` when you export a view under a name you choose, the temporary file used while saving (see [How a file is written](#how-a-file-is-written)), `views.toml` in the config folder (`$XDG_CONFIG_HOME/mdgrid/`, or `~/.config/mdgrid/` if that is not set), and the view-state file in the state folder (`$XDG_STATE_HOME/mdgrid/`, or `~/.local/state/mdgrid/`). It reads `config.toml` but never writes it.

How values are written:

- A new value keeps the quoting style of the old one (plain, `'single'`, or `"double"`). If the old quoting cannot represent the new value safely, mdgrid uses double quotes. A value containing a line break is rejected.
- Strings that YAML could read as something else, such as `no`, `yes`, `on`, numbers, values containing `: ` or ` #`, or values starting with a symbol, are quoted, so they read as the same string under both YAML 1.1 and YAML 1.2.
- In a date or datetime column, a date is written unquoted (`due: 2026-11-03`), as Obsidian does, unless the old value was quoted, in which case the quotes are kept. The exception is a quoted empty string (`due: ""`), which is replaced by an unquoted date. In a text column, a date-like string is quoted, because writing it unquoted would change its type.
- If you change a value and then change it back to what was last read from the file, the cell stops being a pending change and nothing is written for it. Re-writing the "same" value could still change its quoting or format, so mdgrid does not touch it.
- A list keeps its original form (one-line `[a, b]` or one item per line) and indentation, and items you did not change keep their original spelling. Removing every item writes `key:` and keeps the key.

## Read-only notes

Some notes cannot be written without risking damage, or without guessing which value is meant. mdgrid shows these notes but makes the whole row read-only, and shows the reason when you try to edit a cell. Except for the two forms that depend on `add_frontmatter`, this cannot be changed by any setting.

Each reason below has a small example note. In the examples, every line ends with LF unless marked, and these markers stand for bytes:

- `<BOM>` is the UTF-8 byte order mark (EF BB BF).
- `<CRLF>` at the end of a line means the line ends with CR LF instead of LF.
- `<XX>`, two uppercase hex digits, is that single byte (for example `<E9>`).

### `no_frontmatter`

The first line is not `---`, so the note has no frontmatter.

```text
Meeting notes
Talked about the release date.
```

By default this note is writable: saving a value adds a frontmatter block at the top, and the body stays as it was. With `add_frontmatter = false` in the config file, these notes are read-only instead and show the reason.

A note whose first line is `+++` (TOML frontmatter, as used by Hugo and Zola) is always read-only, whatever the setting: adding a YAML block above it would break the page for the site generator.

### `empty_frontmatter`

The frontmatter is just the opening and closing `---`, with nothing between them.

```text
---
---
Meeting notes
```

By default this note is writable: saving a value adds the key line between the two `---` lines. With `add_frontmatter = false` in the config file, these notes are read-only instead and show the reason. Frontmatter that contains only blank lines or comments is not empty; it is treated as normal frontmatter.

### `bom`

The file starts with a UTF-8 byte order mark.

```text
<BOM>---
status: todo
---
Meeting notes
```

The values are still shown (and printed by `--print`) when the frontmatter after the byte order mark can be read; only writing is refused.

### `not_utf8`

The file is not valid UTF-8, for example a note saved in Latin-1 or Shift_JIS.

```text
---
title: Caf<E9>
---
Meeting notes
```

### `mixed_newlines`

The frontmatter mixes line endings (CRLF and LF), or has a stray CR inside a line. Only the lines from the opening `---` to the closing `---` are checked; the body may mix line endings freely.

```text
---<CRLF>
status: todo
---<CRLF>
Meeting notes
```

### `duplicate_key`

The same top-level key appears more than once, so there is no single value to edit.

```text
---
status: todo
status: done
---
Meeting notes
```

### `unclosed`

The frontmatter opens with `---` but never closes.

```text
---
status: todo
Meeting notes
```

### `invalid_yaml`

The frontmatter is not something mdgrid can read safely as YAML. A common case is an unquoted value containing `: `, which YAML rejects. Other cases include a flow list that is never closed and has no continuation lines (one that continues onto the next lines only makes that value read-only; see [Read-only values](#read-only-values)), a tab used for indentation, or an indented line before the first key.

```text
---
title: Re: release date
---
Meeting notes
```

Frontmatter written as one JSON or flow-style map (`{"status": "todo"}` between the `---` lines) is also read-only, but its values are shown (and printed by `--print`, and used by `file.tags`).

### `hard_link`

The note file has more than one hard link (on Unix). Saving replaces the file with a new one, which would separate this name from the other links, so mdgrid does not write it. This is found from the file system rather than from the note's content, so there is no example note: the row is marked read-only when the note is loaded, and the check is repeated when saving, including right before the file is replaced.

## Read-only values

Within a note that can be written, some individual cells are still read-only, each with its own reason:

- Columns that are not frontmatter keys: file properties (`file.*`) and formula columns (`formula.*`).
- Nested maps (`key:` followed by indented `sub: value` lines, or `{a: 1}`) and block scalars (`|` and `>`). mdgrid shows them but does not rewrite them.
- Values that use anchors or aliases (`&a`, `*a`), tagged values (`!!str 12`), and values that span several lines (a plain or quoted scalar continued on the next line, or a flow list like `[a,` followed by ` b]`).
- A column whose type differs between the `types.json` files of different vault roots.
- Lists that mdgrid cannot rewrite while keeping their original layout. In a list column, the item picker does not open for these, and the reason names the form: a blank line or a comment between items, a comment at the end of an item, items with different indentation, a tab after `-`, an item that continues onto the next line, nested items (a list or map inside an item), items that are not strings (for example `- 2024`, which would change type if rewritten). A list column written as a single string (`tags: x`) opens as a one-item list and is rewritten as `tags: [x, y]` when you change it; a single value that is not a string (`tags: 2024`) stays read-only.

For example, `notes` and `aliases` here are read-only, while `status` can be edited:

```yaml
---
status: todo
notes: |
  First line
  Second line
aliases:
  - one

  - two
---
```

## External changes

A note may change on disk while mdgrid has it open, for example when you edit it in Obsidian or with `e` in your editor, or when a sync tool updates it.

- **Detection.** mdgrid polls the files' modification time and size (every `poll_ms`, 1000 ms by default). A note with no pending changes is simply re-read. A note with pending changes is also re-read so the table shows the new values, but your pending changes are kept and the row is marked as changed outside (`!`). If a pending value now equals the value in the file, it stops being a pending change.
- **The save baseline.** For each note, mdgrid remembers a baseline: the modification time, size, and SHA-256 hash of the content it had last read at the moment you first made a pending change to that note. Re-reading after an external change does not move the baseline. After mdgrid writes a note itself, the content it wrote becomes the new baseline, so saving the same note twice is not mistaken for an outside change.
- **Saving stops.** Right before writing, mdgrid reads the note again. If the modification time, size, or hash differs from the baseline, it does not write that note. The other notes are still saved. The stopped note keeps its pending changes, and the review screen shows its diff and marks it as changed outside.
- **Write over, or discard.** For a stopped note you can choose `o` to write over the external change, or `d` to discard your pending changes for that note (the file keeps the external change). Writing over re-reads the file and uses that content as the new baseline only if it is the same content the diff showed you. Then only your edited values are written on top, so the external change to other keys and the body is kept. If the file changed again after you saw the diff, nothing is written and the diff is rebuilt for you to review. You can also go back (`Esc` or `q`) and keep the pending changes for later.

mdgrid also marks conflict files from sync tools (names containing `.sync-conflict-` or `conflicted copy`), so you can notice them and resolve them by hand.

## How a file is written

Saving a note follows these steps. If any step fails, the original note is left untouched, the temporary file is removed, and the pending changes are kept.

1. **Check the baseline.** mdgrid resolves symbolic links and works on the target file. It refuses a note with more than one hard link or one you have no permission to write, reads the note, and stops if it no longer matches the baseline.
2. **Build and verify the new content in memory.** It replaces only the edited value ranges, then reads the result back and checks that each edited key has the intended value, that every other key has the same value and form as before, that no unexpected key appeared, and that the body is unchanged. If any check fails, nothing is written.
3. **Write a temporary file in the same folder.** The temporary file is created next to the note (a hidden name starting with `.` and containing `.mdgrid-tmp.`), so the final rename stays within one file system. The note's permissions are copied to it, and on Unix its owner and group as well.
4. **Write, fsync, and read back.** The new content is written to the temporary file and flushed to disk with `fsync`. mdgrid then reads the temporary file back and checks that it matches the verified content byte for byte.
5. **Check again right before replacing.** mdgrid checks the hard link count, the write permission, and the baseline once more. If the note changed in the meantime, it stops.
6. **Replace by renaming.** The temporary file is renamed over the note. The rename is atomic, so the note is always either the old version or the new one, never half written. mdgrid then fsyncs the folder as well, where the file system supports it.

Some consequences:

- **Symbolic links** stay links: mdgrid writes to the file the link points to, and the link itself is unchanged.
- **Permissions** are kept: the new file has the same mode (and, on Unix, owner and group) as the old one. If they cannot be copied, the save fails. Only these are kept: because the note is replaced by a new file, extended attributes, ACLs, and the creation time are those of the new file (so the creation time Obsidian shows may change).
- **Notes without write permission** (for example mode `0444`) are read-only. Replacing by rename needs write permission only on the folder, so mdgrid checks the file itself: if you cannot write to it (mdgrid checks whether the file can be opened for writing, then closes it without writing, so `root` can still write; on Linux, tools that watch files, for example through inotify's `IN_CLOSE_WRITE`, may react to this), every cell of that note is read-only and selecting one shows the reason "書き込めない権限" (no write permission); setting a value on many rows skips that note and counts it. If the permission is removed after you have pending changes for the note, saving stops for that note without writing (its content and mode stay the same), shows the same reason, and keeps its pending changes; other notes are still saved. If the folder itself is not writable, the temporary file cannot be created, so the save fails and the note is unchanged.
- **The new baseline** is taken from the temporary file and the verified content before the rename, so a change made by someone else between the rename and the next read is still detected as an external change.

To look at notes without any risk of writing, start mdgrid with `--readonly`.
