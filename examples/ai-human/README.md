# ai-human: tasks for people and tasks for agents

When AI agents do part of the work, tasks split in two: those an agent can finish on its own, and those where a person has to do the last step (pay, sign, submit, send, or anything away from the computer). This sample puts the two in separate groups, people first, with one `.base` and one config.

`Tasks/` holds 11 admin tasks of Northwind Studio, the made-up company of the [other samples](../README.md): 9 open and 2 finished, dated October 2026.

## Open it

From the repository root, copy it to a temporary folder first (then you can edit and save):

```sh
cp -R examples/ai-human /tmp/mdgrid-ai-human
mdgrid --config /tmp/mdgrid-ai-human/config.toml /tmp/mdgrid-ai-human/Tasks.base
```

The first view, People and AI, lists the tasks under `👤 Needs a person` and `🤖 For an agent`. Enter on a heading folds it. `[` `]` switch views.

| View | Shows |
|---|---|
| People and AI | Open tasks, people first; by priority, then due date, inside each group |
| People only | The tasks that need a person, by due date |
| AI only | The tasks an agent can finish, by priority, with the agent and the folder it runs in |
| Done | Finished and dropped tasks, with who did them |

As text instead of a screen:

```sh
mdgrid /tmp/mdgrid-ai-human/Tasks.base --print --format md --with-path --view "People only"
```

## How a task is sorted

The formula `Owner` in `Tasks.base` decides:

```yaml
formulas:
  Owner: if(actor == "human" || if(human_gate, human_gate != "none", false), "👤 Needs a person", "🤖 For an agent")
```

- `actor: human`: work away from the computer (picking something up, going to a counter). A person.
- `human_gate` is `submit`, `pay`, `sign` or `send`: the agent prepares, a person presses the last button. A person.
- Anything else is an agent task, including a note with no `actor`.

The People and AI view groups by this formula. `👤` sorts before `🤖`, so ascending order (`ASC`) puts the people's group on top.

## Frontmatter

| Key | Column | Example values |
|---|---|---|
| `status` | Status | `todo`, `doing`, `done`, `dropped` |
| `priority` | P | `p1` to `p5` |
| `due` | Due | `2026-10-15` |
| `actor` | (used by the formula) | `ai`, `human` |
| `human_gate` | Human gate | `none`, `submit`, `pay`, `sign`, `send` |
| `runner` | Agent | `Claude`, `Codex` |
| `workdir` | Workdir | `~/src/clinic-booking` |
| `done_when` | (not shown) | One sentence: when the task is done |

## The config (config.toml)

- `[new_note]`: a new task (`a`) asks `actor`, `human_gate` and `due`, and starts with `status = "todo"` and `priority = "p3"`.
- `[display]`: lines between columns, zebra rows, and an empty line above the second group heading (`group_gap`).

## Make it your own command

Put a `Tasks.base` in your own task folder and add an alias:

```sh
alias tasks='mdgrid --config ~/notes/mdgrid-tasks.toml ~/notes/Tasks.base'
alias mine='mdgrid ~/notes/Tasks.base --view "People only"'
```

日本語: AI エージェントに任せきれるタスクと、最後に人の手が要るタスク(払う・署名する・提出する・送る、PC の外の用事)を、`.base` の式1つと設定1つで分けて、人のタスクを上に並べる見本。`mdgrid --config examples/ai-human/config.toml examples/ai-human/Tasks.base --readonly` で開く。
