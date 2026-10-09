# How mdgrid sees your notes

[日本語](../ja/concepts.md) · [Contents](index.md)

This page explains the few ideas the rest of the manual builds on: tables, views, registered tables, links, workspaces and the relation map. Once they click, every screen of mdgrid is one of these seen from a different side.

- [Notes become a table](#notes-become-a-table)
- [Views: different ways to look at the same notes](#views-different-ways-to-look-at-the-same-notes)
- [Nothing is written until you save](#nothing-is-written-until-you-save)
- [Registered tables: bookmarks you switch between](#registered-tables-bookmarks-you-switch-between)
- [Links turn folders into linked tables](#links-turn-folders-into-linked-tables)
- [Workspaces: which tables belong together](#workspaces-which-tables-belong-together)
- [The relation map](#the-relation-map)
- [Where mdgrid keeps things](#where-mdgrid-keeps-things)

## Notes become a table

Open a folder and every Markdown note in it (and in its subfolders) becomes a **row**. Every frontmatter key that any of the notes uses becomes a **column**. The note's file name is shown on the left; it is the row's key, so there is no id column to maintain.

```
Tasks/                          Note            status   due         tags
├── Fix login.md      ──►       Fix login       doing    2026-10-05  [bug, web]
├── Ship v0.2.md                Ship v0.2       todo     2026-10-10  [release]
└── Write docs.md               Write docs      done     2026-10-01  [docs]
```

mdgrid looks at the values of a column and decides what kind it is: text, number, date, checkbox or list. The kind decides how a cell is edited (a calendar for dates, a list of choices for text that repeats, checkboxes for lists) and how it is shown. A value that does not fit the column's kind is shown as it is with a `!` in front, never changed.

A key that a note does not have shows as an empty cell; an empty string shows as `""` and a null as `∅`, so you can always tell the three apart.

## Views: different ways to look at the same notes

A **view** is a way of looking at the table: which columns, in which order, which rows (filters), how they are sorted and grouped. The tabs under the header are the views of the table you opened.

- **All notes** is always there when you open a folder: every note, every key.
- **`.base` views**: when you open an Obsidian Bases `.base` file, its table views become the tabs. mdgrid reads them but never changes the `.base` file.
- **mdgrid views**: in **View settings** (`o`) choose columns, filters, sorting, grouping and display options, then **Save as** a name. The view becomes a new tab. Saved views live in `views.toml` in the config folder, never next to your notes.

Besides saved views, mdgrid quietly remembers how you left each table — column widths, hidden columns, the selected view — and puts it back the next time you open the same folder or `.base`.

## Nothing is written until you save

Every edit is **pending** at first: the cell gets a `*` and the bottom band counts the unsaved changes. `Ctrl+S` shows a diff of each file before anything is written, and `Enter` writes them. Only the value you changed is rewritten (or one line is added for a new key); the rest of the file — other keys, their order, comments, line endings, the body — stays byte for byte the same. See [Write-back safety](../../safety.md) for the exact rules.

## Registered tables: bookmarks you switch between

**Register** a table you use often under a name and a group (**register this table** in the palette `:`). Plain `mdgrid` with no arguments then lists them, and **open a registered table** switches between them. Registrations are bookmarks in `places.toml`; they do not change the notes.

A shell alias is the other way to reach a table quickly: `alias tasks='mdgrid ~/notes/Tasks'` makes `tasks` a command of its own.

## Links turn folders into linked tables

A frontmatter value can point at another note:

```yaml
project: "[[mdgrid]]"                 # wiki link: the note named mdgrid
assignee: "[Alice](../members/Alice.md)"   # Markdown link
related: [../projects/Website]       # a plain path also works
```

mdgrid shows a link by the target's name, lets you pick a target by name when you edit the cell, opens the target (`:open_link`, or `x` on the cell), and lists the notes that link to the current one (`:linked_rows`). A link whose note does not exist is shown with a `?` in front.

Because the file name is the key, a folder of notes works like a table and links work like foreign keys. A list of links stands in for a many-to-many join table.

```
 tasks ──project──► projects ──lead──► members
   └──────assignee──────────────────────┘
```

## Workspaces: which tables belong together

To resolve a link by name, mdgrid has to know which tables to look in. A **workspace** answers that: a named set of tables that belong together, such as the tasks, projects and members of one project. Links, linked rows, the relation map and the tabs at the top right use only the tables of the current workspace, so your work vault and your reading notes do not mix.

mdgrid decides the workspace of the table you open in this order:

1. `-w <name>` on the command line.
2. A marker file `.mdgrid/workspace.toml` in the folder or above it. This is useful in a repository you share: the workspace travels with the notes.
3. A workspace in `workspaces.toml` (in the config folder) that contains the table. Manage these from the palette or with `--add-to`, `--remove-from` and `--workspaces`.
4. Detection: the nearest Obsidian vault (a folder with `.obsidian/`), or a git repository if you turn that on with `workspace_detect`. Its folders of notes become the tables.
5. Otherwise, the registered tables.

The header shows `· workspace <name>` when one is in use. See [Group tables into workspaces](tasks.md#group-tables-into-workspaces).

## The relation map

Press `R` (or click **Relations** in the tabs at the top right) to see the workspace as a diagram: one box per table with its row count and columns, and an arrow from each link column to the table it points at, marked `N:1` (one link) or `N:N` (a list of links). Select a table or a link with the arrow keys or the mouse; the details and the linked records follow the selection. `Enter` (or a second click) opens the table. Press `R` again to go back.

## Where mdgrid keeps things

| What | Where |
|---|---|
| Your notes | Unchanged, except the values you save |
| Settings | `~/.config/mdgrid/config.toml` (`--print-config` prints every item with its default) |
| Saved mdgrid views | `~/.config/mdgrid/views.toml` |
| Registered tables | `~/.config/mdgrid/places.toml` |
| Workspaces | `~/.config/mdgrid/workspaces.toml`, or `.mdgrid/workspace.toml` in a folder |
| How you left each table (widths, hidden columns, view) | `~/.local/state/mdgrid/` |

`XDG_CONFIG_HOME` and `XDG_STATE_HOME` move the two folders. The files are plain TOML you can edit by hand; mdgrid keeps your comments when it updates `places.toml` and `workspaces.toml`.
