---
name: mdgrid
description: Query, filter, sort and export the YAML front matter of Markdown notes (an Obsidian vault, a task folder, a .base view) as CSV, TSV, JSON or a Markdown table with the mdgrid CLI, and write edited values back safely. Use when the user asks things like "list my tasks that are todo or doing", "export these notes to CSV", "make a table of notes due this week", "which notes have tag X", or wants a file of note properties to hand to another AI or spreadsheet.
---

# mdgrid

mdgrid reads the front matter of Markdown notes as a table: one row per note, one column per key. It also opens Obsidian Bases `.base` files and their views, and works without Obsidian running. Use its CLI instead of parsing Markdown files yourself: it reads YAML the same way every time, understands Obsidian's expression language, and never changes a note unless you pass `--apply ... --yes`.

Check that it is installed with `mdgrid --version`. If it is missing, tell the user it can be installed from https://github.com/S6U5/mdgrid (do not install it without asking).

## Export a table

```text
mdgrid <folder | .base | .md> ... --print [--format csv|tsv|json|md] [--with-path]
       [--view <name>] [--filter <expression>]... [--sort <column>[:desc]]...
```

- `--print` writes the table to standard output and exits; nothing on screen, no notes changed. Redirect it to a file with `>`.
- `--format`: `csv` (default) for spreadsheets, `md` for a Markdown table to read or paste into a chat, `json` for scripts (a list of objects), `tsv` for tab-separated.
- `--with-path` adds a first column `path` with each note's path. Add it whenever the result may be edited and applied back, or when the reader needs to open the notes.
- `--filter` keeps rows where the expression is true. Repeat it to require all of them.
- `--sort column` sorts ascending, `column:desc` descending. Repeat it for tie-breakers.
- With a `.base`, `--view <name>` picks the view (its filters, columns and sort apply; `--filter` narrows it further). Without `--view`, the first view is used.
- Paths are folders (all notes below them), `.base` files, or single `.md` files. Several can be given.

Examples (in a folder `tasks/` whose notes have `status`, `due` and `priority`):

```sh
# Tasks that are todo or doing, as a Markdown table with paths
mdgrid tasks --print --format md --with-path --filter 'status == "todo" || status == "doing"' > active.md

# Everything except done tasks, highest priority first, as CSV
mdgrid tasks --print --filter 'status != "done"' --sort priority:desc > open.csv

# Tasks due within the next 7 days, soonest first, as JSON
mdgrid tasks --print --format json --with-path --filter 'due <= today() + "7d"' --sort due > due-soon.json

# A view of an Obsidian .base file
mdgrid tasks.base --view Open --print --format md > open-view.md
```

Put the expression in single quotes so the shell keeps the double quotes inside it.

## Expressions

`--filter` takes the same expressions as Obsidian Bases filters:

- A front matter key by name: `status`, or `note["key with spaces"]`. A missing key is `null`.
- Comparisons and logic: `==` `!=` `<` `<=` `>` `>=` `&&` `||` `!`, parentheses.
- Strings in quotes: `status == "doing"`. Dates written `YYYY-MM-DD` compare as dates: `due < "2026-11-01"`, `due <= today() + "7d"`, `due >= today() - "2 weeks"`.
- Lists and strings: `tags.contains("work")`, `title.lower().contains("draft")`, `tags.isEmpty()`, `!status.isEmpty()`.
- Files: `file.inFolder("projects")`, `file.hasTag("urgent")`, `file.hasProperty("due")`, `file.name`, `file.mtime > now() - "1d"`.

The full list of operators, functions and methods is in https://github.com/S6U5/mdgrid/blob/main/docs/obsidian-bases.md. If an expression cannot be read, mdgrid prints the reason on one line and exits with code 2; fix the expression rather than falling back to parsing files yourself.

Find the available keys first when you don't know them: `mdgrid <folder> --print --format md` shows every key as a column (pipe it through `head` for large folders).

## Write values back

To change front matter values in bulk, export with `--with-path`, edit the file, then apply it. `--apply` reads CSV or JSON in the `--print --with-path` form and changes only the cells that differ.

```sh
# 1. Export the rows to change, with paths
mdgrid tasks --print --with-path --filter 'status == "doing"' > doing.csv

# 2. Edit doing.csv (here: mark them done)
sed 's/,doing,/,done,/' doing.csv > done.csv

# 3. Show the diff; nothing is written yet
mdgrid tasks --apply done.csv
```

Edit the file any way you like (by hand, a script); keep the `path` column and the header, and don't rely on fixed column positions, which follow the keys of the notes. Step 3 prints a unified diff of the notes and writes nothing. **Show that diff to the user and wait for their approval**, then run the same command with `--yes` (`mdgrid tasks --apply done.csv --yes`) to write. Never add `--yes` on your own. mdgrid writes only the changed values and leaves the rest of each note byte for byte as it was.

## Handing results to another AI or a person

For a chat assistant or a person, `--format md` gives a readable table; attach the file or paste it. For a spreadsheet use `csv`. Keep `--with-path` if they should be able to find the notes, and drop it to share less.
