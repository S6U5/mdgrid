# mdgrid

[日本語](../ja/index.md)

mdgrid shows a folder of Markdown notes with frontmatter as a table in the terminal and lets you edit the values in place. Each note is a row and each frontmatter key is a column. It works on an Obsidian vault without opening Obsidian, and it also opens the table views of Obsidian Bases `.base` files.

![A folder opened in mdgrid](images/table.svg)

It suits you if:

- you track tasks, reading lists and the like in frontmatter (in Obsidian or elsewhere)
- you want to change `status` or `due` on many notes without opening each one in an editor
- you do not want an edit to disturb the rest of the file — key order, comments, the body

mdgrid writes only the value of the key you changed (adding a key line when the key is missing) and leaves every other byte as it was. Before writing, it shows you a diff per file to confirm.

## Contents

- [Getting started](getting-started.md) — build, first run on the sample vault, reading the screen, the first keys
- [Task guides](tasks.md) — edit and save, find and filter, edit many rows, create notes, save a view, use with other commands, choose a theme

## Reference

The exact rules live in reference pages whose content the tests check.

- [Key bindings](../../keys.md) — every key in every mode, and rebinding
- [Configuration](../../config.md) — every item of the config file
- [Write-back safety](../../safety.md) — what is written and what is not, notes that open read-only and why
- [Obsidian Bases support](../../obsidian-bases.md) — which parts of `.base` mdgrid reads
- [Themes](themes.md) — samples of the seven color themes and how to choose one
- [Screen catalogue](shots.md) — a screenshot of every screen
