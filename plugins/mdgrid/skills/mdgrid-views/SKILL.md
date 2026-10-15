---
name: mdgrid-views
description: Set up how mdgrid shows a folder of Markdown notes by writing its marker file `.mdgrid/workspace.toml` — which table opens, the views (filters, formula columns, column names, grouping, tabs by status), column types, allowed values for choice columns, and how the note column shows names — then verify it with mdgrid itself. Use when the user asks to "set up mdgrid for this folder", "make tabs by status", "show only todo and doing", "hide the date in the file names", "add a column", "make due a date", "give status a dropdown", or to fix a folder that opens with the wrong rows (README, AGENTS.md) in mdgrid.
---

# mdgrid-views

mdgrid reads a marker file `.mdgrid/workspace.toml` in a folder. With it, opening the folder (`mdgrid` there, `mdgrid .`, or its path) opens a chosen table with its views instead of every note in the folder. This skill writes that file carefully and checks the result. Read the whole checklist before writing; every item comes from a mistake that was made before.

Check that mdgrid is installed with `mdgrid --version`. If it is missing, tell the user it can be installed from https://github.com/S6U5/mdgrid (do not install it without asking).

## Checklist (do all of it)

1. **Find the data's own spec first.** If the notes follow a written format (a spec, a schema, a lint config, a README that lists the fields), take every column's type and every choice column's values from it. Do not guess what the spec already says.
2. **Without a spec, infer from the data and stay conservative.** Look at the values (`mdgrid <folder> --raw --print --format md | head -50`). A column whose values all read as `YYYY-MM-DD` is a `"date"`; `YYYY-MM-DDTHH:MM` is `"datetime"`; a column with a few repeated values is a choice column. When unsure, leave the column untyped (it stays text) and ask the user. Untyped is valid; a wrong type marks good values as problems.
3. **Type every date column** in `[types]` (`"date"`, `"datetime"`, `"number"`, `"checkbox"`, `"list"`, `"text"`). Typed dates sort and edit as dates and get a `!` when a value doesn't fit.
4. **Give every choice column its values** in `[rules.<column>] values = [...]`. They become the edit candidates (even values no note uses yet) and anything else gets a `!`. When a column means different things in different tables (a task's `status` vs. an event's `status`), put the rule under that table (`[table.rules.status]`), not at the top.
5. **Never hide what identifies a row.** If the note column's name is noisy, trim it (`note_path`, `note_name`) instead of hiding it (`note_path = "off"` hides names; use it only when another visible column, such as a title, identifies every row — and say so to the user).
6. **Write tabs by status as `where` arrays**, not as `status == ...` chains: `where = { status = ["todo", "doing"] }`. One tab can hold several values.
7. **Quote non-ASCII keys** in TOML: `"担当" = '...'`, `"note.title" = { displayName = "題名" }`. An unquoted non-ASCII key makes the whole marker unreadable (mdgrid then ignores it).
8. **Verify before saying it is done** (below). Look at the result the way the user will: the right rows, the names readable, no stray files (README, AGENTS.md) in the table.

## The marker file

```toml
# notes/.mdgrid/workspace.toml
name = "tasks"
open = "Ledger"                  # the table that opens for this folder; `mdgrid --raw` opens the folder itself

[display]
group_gap = true                 # an empty line above every group heading but the first
note_path = "name"               # note column: "common" (default), "name", "full" or "off"
note_name = '^\d{8}_'            # regex on the name: removes the match, or shows the group `name` / the first group

[types]
due = "date"
done_at = "date"

[rules.priority]
values = ["p1", "p2", "p3"]

[[table]]
name = "Ledger"

# The same content as an Obsidian .base file, written as TOML (no .base file needed).
[table.base]
filters = { or = ['file.inFolder("todo")', 'file.inFolder("archive/todo")'] }

[table.base.formulas]
owner = 'if(actor == "ai", "AI", "Human")'

[table.base.properties]
"note.title" = { displayName = "Title" }
"formula.owner" = { displayName = "Owner" }

[[table.base.views]]
type = "table"
name = "Open"
where = { status = ["todo", "doing"] }
order = ["title", "status", "due", "priority"]
groupBy = { property = "formula.owner", direction = "ASC" }
sort = [{ property = "due", direction = "ASC" }]

[[table.base.views]]
type = "table"
name = "Done"
where = { status = ["done"] }
order = ["title", "done_at"]

[table.rules.status]
values = ["todo", "doing", "done"]

[[table]]
name = "Archive"
path = "archive"                 # a plain folder table; switch tables with `w` or the breadcrumb
```

- `[table.base]` takes the same keys as a `.base` file (`filters`, `formulas`, `properties`, `summaries`, `views`) with the same expressions; see https://github.com/S6U5/mdgrid/blob/main/docs/obsidian-bases.md. A table can instead point at a real `.base` with `path = "x.base"` (Obsidian then sees the same views); don't write both.
- `where` works at the top of `[table.base]` (every view) and in each view; keys must all match, values are "any of", list columns match by element, and it combines with `filters`.
- A `where` key can be a formula column (`"formula.due_bucket" = ["overdue", "today"]`). Define a classification once in `formulas` (for example a due-date bucket with `today()`) and let each tab pick buckets, instead of repeating date conditions per tab.
- Settings under `[display]`, `[types]`, `[rules.*]` at the top apply to every table of the folder; the same sections under a `[[table]]` (`[table.display]`, `[table.rules.status]`) apply to that table only.
- The full list of settings is in https://github.com/S6U5/mdgrid/blob/main/docs/config.md.

## Verify

Run these in the folder and fix anything they report before telling the user it is done:

```sh
mdgrid --print-config --resolved     # warnings start with "mdgrid:"; each setting shows where it came from
mdgrid --print --format md | head    # the first view's rows: right rows, no README/AGENTS.md
mdgrid --print --view Done | head    # each view you wrote
mdgrid --check                       # values that don't fit a type or a rule (exit 1 if any)
```

If `--print` still shows every note, the marker was not read: look for a TOML error (often an unquoted non-ASCII key) in `mdgrid --print-config --resolved`. Finally, open `mdgrid` and look at the screen (or ask the user to): the tabs, the groups, and the note names.
