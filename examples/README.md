# Samples

Every sample here belongs to the same made-up company, **Northwind Studio**: a small web studio with five people, three clients and four projects. Open them from the repository root. Saving writes the files, so either copy a sample to a temporary folder first or add `--readonly`.

| Sample | What it shows |
|---|---|
| [studio/](studio) | The studio's tasks, projects, clients, people and meetings as five linked tables, with a `.base` of views, a workspace marker, column types and a CSV file. Start here. |
| [quirks/](quirks) | Notes that are awkward to edit: no frontmatter, an empty one, a key written twice, a sync conflict, values of the wrong type, nested maps. What mdgrid shows and why it will not break them. |
| [configs/](configs) | A config that turns many features on, for the studio. |
| [ai-human/](ai-human/README.md) | The studio's admin tasks split into those an AI agent can finish and those that need a person. |
| [places/](places) | Registered tables: the studio's tables and the other samples in one list. |
| [themes/](themes) | One config per color theme. |

`sh demos/try.sh <sample>` opens a sample in a throwaway copy with its own config, and removes it when you quit.

The dates sit around early October 2026. The screens in the README and the manual are taken as if today were 2026-10-06; set `MDGRID_TODAY=2026-10-06` to see the same overdue and "this week" rows.

## studio: linked tables

| Folder | Notes | Links to |
|---|---|---|
| `Tasks/` | 20 tasks | `project` (one project), `owner` (one person), `depends_on` (a list of other tasks) |
| `Projects/` | 4 projects | `client` (one client), `lead` (one person), `team` (a list of people) |
| `Clients/` | 3 clients | `account_manager` (one person) |
| `People/` | 5 people | nothing; everything points here |
| `Meetings/` | 7 meetings | `project` (one project), `attendees` (a list of people) |

A link is the note's file name: `project: "[[Clinic Booking App]]"` points at `Projects/Clinic Booking App.md`. There is no id column and no join table: a many-to-many link (`team`, `attendees`, `depends_on`) is a list of links. Every link in the sample has a note to point at (a test checks it).

```sh
cp -R examples/studio /tmp/studio
mdgrid /tmp/studio/Tasks.base         # the tasks with their views
mdgrid /tmp/studio/Projects           # any folder is a table too
mdgrid /tmp/studio/equipment.csv      # so is a CSV file
```

Things to try:

- **Links show by name.** The `project` and `owner` cells show `Clinic Booking App` and `Leo Park`, not the `[[...]]` written in the file.
- **Pick a link.** Enter on an `owner` cell lists the notes in `People/`; the one you pick is written as `[[name]]`.
- **Follow a link.** On a link cell, `:open_link` (or "Open the link target" in the `x` menu) opens the target's table with its row selected: from a task to its project, from the project to its client.
- **Linked rows.** On a project, `:linked_rows` lists the tasks and meetings that point at it, with the column they point from. On a person, it lists everything they own, lead, attend or manage.
- **The relation map.** Press `R` for the five tables as boxes and an arrow per link column: `N:1` for one link, `N:N` for a list. `depends_on` points back at `Tasks` itself.
- **Views.** `Tasks.base` has seven views (`[` `]` switch them): Open (the total estimate and the earliest due date under the table), This week (a formula marks overdue and soon), By status, By project, By owner, Dependencies and Workload (hours from the estimate).

The other files: `.mdgrid/workspace.toml` makes the five tables one workspace with no setup (links, linked rows, the relation map and the `w` list look at these tables). `.obsidian/types.json` declares the column types, the same file Obsidian uses. `Tasks.base` is an Obsidian Bases file. `equipment.csv` is the studio's equipment list.

## quirks: notes that are awkward to edit

The studio's office chores, kept in notes with every odd shape mdgrid has to handle. Open the folder (`mdgrid examples/quirks --readonly`) and look at what each one does:

| Note | What is odd | What mdgrid does |
|---|---|---|
| `Notes/Snack list.md` | No frontmatter | Editable; saving adds a frontmatter at the top (`[edit] add_frontmatter`, on by default) |
| `Notes/Blank.md` | An empty frontmatter (`---` twice) | Editable; saving adds the key between the lines |
| `Notes/Weekly review.md` | `status` written twice | The whole row is read-only, with the reason on the bottom line |
| `Notes/Ideas.sync-conflict-20260930-1.md` | A sync conflict copy of `Ideas.md` | Marked, so you can tell it from the original |
| `Tasks/Return the library books.md`, `Tasks/Book the team lunch.md`, `Tasks/Clean up the contact list.md` | `status: null`, `status: ""` and no `status` key | Three different cells; all three fall into one empty group |
| `Tasks/Clean up the contact list.md`, `Tasks/Service the studio bikes.md` | A nested map (`details`), a block scalar (`steps: \|`) | Shown, but read-only |
| `Tasks/Pack for the offsite.md`, `Tasks/Get quotes for new chairs.md`, `Tasks/Collect the tax documents.md` | `priority: high`, `due: someday`, `done: later` | The value stays as written, marked as the wrong type for the column |
| `Tasks/Write up the offsite notes.md` | A value with line breaks | Shown on one line until you open it |

`Tasks.base` adds views for the edge cases of Obsidian Bases: a formula mdgrid does not support (the `Unsupported` view shows a dim `?`) and a `cards` view (not opened; mdgrid shows table views only). With the config in [configs](configs), `mdgrid --config examples/configs/studio.toml examples/quirks/Tasks --check` lists the values that break the rules.

## configs: a config with many features on

[configs/studio.toml](configs/studio.toml) is made for the studio. Start mdgrid with it:

```sh
mdgrid --config examples/configs/studio.toml examples/studio/Tasks.base --readonly
```

| Item | Value | Where you see it |
|---|---|---|
| `[dates] format` | `"YYYY/MM/DD (ddd)"` | Dates show as `2026/10/06 (Tue)`, and you type them that way (notes keep `YYYY-MM-DD`) |
| `[dates] week_start` | `"mon"` | The calendar starts on Monday |
| `[display]` | `search_bar`, `column_lines`, `zebra`, `group_gap` | The search field above the table, lines between columns, zebra rows, a gap above group headings |
| `[new_note]` | asks `project`, `owner`, `due`, `priority`; sets `status = "todo"` | `a` opens a form for a new task |
| `[rules.status]` | required, one of `todo`, `doing`, `done` | Other values get a `!`; `--check` lists them |
| `[look.colors.values]` | `todo`, `doing`, `done` | The status values get their own colors |
| `[keys.table]` | `"ctrl+f" = "search"` | `Ctrl+F` searches, like `/` |

## places: registered tables

`XDG_CONFIG_HOME=examples/places mdgrid` starts with the registered tables in [places/mdgrid/places.toml](places/mdgrid/places.toml): the studio's tasks (on the Open view), projects, meetings and equipment, and the other samples.

日本語: 見本はどれも、架空の小さな制作会社 Northwind Studio のもの。主役の [studio](studio) は、タスク・案件・顧客・人・会議の5つの表をフロントマターのリンクでつないだルートフォルダ。[quirks](quirks) は扱いにくいノート(フロントマター無し・重複キー・型の合わない値など)の見本、[configs](configs) は多くの機能を有効にした設定、[ai-human](ai-human/README.md) は人と AI のタスクを分ける見本。どれもリポの根で開き、保存するなら一時フォルダに写すか `--readonly` を付ける。
