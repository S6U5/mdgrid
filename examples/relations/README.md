# relations: tables linked by notes

Three folders of notes that point at each other with links in the frontmatter:

- `tasks/`: each task has `project` (one project), `assignee` (one member) and `related` (a list of projects).
- `projects/`: each project has a `lead` (one member).
- `members/`: people.

A link is the note's file name: `project: "[[mdgrid]]"` points at `projects/mdgrid.md`. There is no id column, and no join table for many-to-many: `related` is a list of links. `[text](../projects/mdgrid.md)` and a plain path such as `../projects/mdgrid` work too.

Try it from the repository root (copy it first if you want to save edits):

```sh
mdgrid examples/relations/tasks
```

- Link cells show the target's name. A link whose note does not exist is shown with `?`.
- Enter on the `project` cell lists the notes in `projects/` by name; the one you pick is written as `[[name]]`.
- `:open_link` (or `x` on the cell) opens the target. `:linked_rows` lists the notes that link to the current one, with the column they link from.
- Register the three folders (`:register_place` in each) to jump between them: opening a target moves to its table and selects its row.

日本語: 3つのフォルダのノートが、フロントマターのリンクで互いを指す見本。リンクはノートのファイル名を指し、id の列も中間の表も要らない。`mdgrid examples/relations/tasks` で開き、`:open_link` で行き先を開き、`:linked_rows` でつながった行を出す。
