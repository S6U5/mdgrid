# mdgrid

[日本語](../ja/index.md) · Read it with the screenshots at https://s6u5.github.io/mdgrid/manual/en/ (the screens are taken there from the real mdgrid and are not kept in the repository)

A tool that opens a folder of Markdown notes as a table in the terminal and lets you edit the values in place.

![Open a folder](images/table.svg)

## When you would want it

You have forty task notes and want to set `status: done` on the finished ones. A few reading notes are missing their `rating`. You would like to look at your bookmarks folder by folder. Too many notes to open one at a time in an editor, and a shame to flatten into a spreadsheet.

Give mdgrid the folder and the notes become rows, the frontmatter keys become columns. `Enter` on a cell edits it; `Ctrl+S` shows the diff and saves. Only the lines with changed values are written. Key order, comments and the body stay as they were. The notes remain plain Markdown, so the same folder stays open in your editor or note app. `.base` views open as they are.

## What it looks like

Besides the table: groups, a folder tree, a map of how the tables link to each other, the counts of a column's values, and one note in detail. The tree, the map, the counts and the detail are one key away; groups come from the view settings. `Esc` brings the table back. They are laid out one screen each in [Ways to see your notes](views.md); what you can do with them is in [What mdgrid can do](features.md).

## Try it on the sample

The quickest way in is an archive from [Releases](https://github.com/S6U5/mdgrid/releases/latest): unpack it and put `mdgrid` on your PATH. With Rust installed, `cargo install mdgrid --locked` works too, but the build takes a few minutes. The samples are in the repository.

```sh
git clone https://github.com/S6U5/mdgrid && cd mdgrid
cp -R examples/studio /tmp/mdgrid-sample
mdgrid /tmp/mdgrid-sample/Tasks
```

If the task table of "Northwind Studio" appears, it worked. `j` and `k` move, `Enter` edits, `q` quits. You are on a copy, so saving never changes the samples. Other ways to install and a tour of the screen are in [Getting started](getting-started.md).

## On your own notes

Four starters are built in: tasks, bookmarks, reading, and people with meetings. Each writes a marker, templates and samples into an empty folder, so you start with a table already shaped.

```sh
mdgrid ~/books --init-workspace --starter reading --with-samples
mdgrid ~/books
```

Starters are there from 0.4.0. See [Create a ledger from a starter](tasks.md#create-a-ledger-from-a-starter). If you already have notes, just pass their folder.

## Contents

- [Ways to see your notes](views.md) — table, view tabs, groups, tree, relation map, value counts, the filter band, themes. One screen each
- [What mdgrid can do](features.md) — by use case (tasks, bookmarks, reading, people and meetings), then editing, saving, bulk edits, finding, creating notes, views, `.base`, links, starters, other commands, read-only, looks. One screen per feature
- [Getting started](getting-started.md) — install, first run on the samples, reading the screen, the first keys
- [Task guides](tasks.md) — edit and save, find and filter, edit many rows, create notes, open a `.base`, save a view, give a folder its own command, register tables, link tables, group tables into workspaces, create a ledger from a starter, use with other commands
- [How mdgrid sees your notes](concepts.md) — tables, views, saving, registered tables, links, workspaces, the relation map, and where mdgrid keeps things
- [Make it look the way you like](appearance.md) — themes, cells as parts or text, frames, what is shown around the table, terminal and fonts
- [When something looks wrong](faq.md) — marks in cells, cells you cannot edit, links with `?`, the relation map, misaligned columns, colors, speed

## Reference

The rules are listed on reference pages whose contents are checked by tests.

- [Key bindings](../../keys.md) — every key in every mode, and rebinding
- [Configuration](../../config.md) — every item of the config file
- [Write-back safety](../../safety.md) — what is written and what is not, notes that open read-only and why
- [Obsidian Bases support](../../obsidian-bases.md) — which parts of `.base` mdgrid reads
- [Themes](themes.md) — samples of the color themes and how to choose one
- [Screen catalogue](shots.md) — a screenshot of every screen
