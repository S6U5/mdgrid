# Key bindings

[日本語](keys.ja.md)

This page lists mdgrid's default key bindings for each mode, along with the action names you use to rebind keys in the config file. Inside mdgrid, `?` shows the same list as a help screen, with the keys you can press right now at the top.

## Rebinding keys

To rebind keys, add a `[keys.<mode>]` table to the config file and write `"key" = "action name"` under it (see [`keys` in the config reference](config.md#keys)). The mode is one of the section headings below, such as `table` or `edit`. The action name comes from the Action column, or from the list at the end of this page.

```toml
[keys.table]
"ctrl+f" = "search"   # Ctrl+F also starts a search
"a" = "none"          # "none" removes a key
"ctrl+n" = "new_note" # so "new note" moves from a to Ctrl+N
```

- Keys look like `"j"`, `"G"`, `"ctrl+s"`, `"shift+tab"`, `"pagedown"`. Modifier and key names are case-insensitive. To bind a shifted letter, write the capital letter (`"G"`). A space separates a key sequence, such as `"g g"`. The key names in the tables below also work as written. For the `\` key, write `"\\"` in a TOML basic string, or `'\'` as a literal string.
- Binding a key that already has an action replaces that action. The action name `"none"` removes a key, so you can free a key and then reuse it for something else.
- You can bind an action only in a mode where it has a default key. One key can't do two things in the same mode, and a single key can't also be the first key of a sequence (`g` alongside `g g`). Remove the conflicting key with `"none"` first.
- Each mode always keeps a way out (for example, `quit` or `palette` in `table`, `cancel` in `edit`). If your config would remove every key for it, mdgrid keeps the default keys and shows a warning.
- Unknown modes, keys, and actions are skipped with a warning.
- Modes where you type text (`edit`, `palette`, `search`, `filter`, `settings_input`, `list_select`, `filter_input`) don't bind single letters by default and don't accept key sequences.
- vim mode (`vim = true` under `[keys]`, see [`keys.vim`](config.md#keysvim)) adds the keys in the "vim mode" part of `table` and the `input_normal` mode. You rebind or remove them like any other key; without vim mode they are not there.

In the `table` mode, full-width letters and symbols typed while a Japanese input method is on act like their half-width keys, and `、` and `・` act like `,` and `/`.

## `table`

The main grid of notes.

When you start mdgrid with `--pick`, `Enter` on a note row prints the marked rows (or the selected row) and quits, `q` cancels, and `Esc` clears the selection, filters, the conditions turned on with digit keys or the notice band, and search as usual, cancelling only when there is nothing left to clear. `Enter` on a group heading row still expands or collapses it, and `e` does not open the editor.

| Key | Action | Description |
|---|---|---|
| `k` | `up` | Move up one row |
| `Up` | `up` | Move up one row |
| `j` | `down` | Move down one row |
| `Down` | `down` | Move down one row |
| `h` | `left` | Move to the column on the left |
| `Left` | `left` | Move to the column on the left |
| `l` | `right` | Move to the column on the right |
| `Right` | `right` | Move to the column on the right |
| `Home` | `first_column` | Move to the first column |
| `0` | `first_column` | Move to the first column |
| `End` | `last_column` | Move to the last column |
| `$` | `last_column` | Move to the last column |
| `PageUp` | `page_up` | Scroll up one screen |
| `PageDown` | `page_down` | Scroll down one screen |
| `Ctrl+u` | `half_page_up` | Scroll up half a screen |
| `Ctrl+d` | `half_page_down` | Scroll down half a screen |
| `g g` | `top` | Go to the first row |
| `G` | `bottom` | Go to the last row |
| `Tab` | `next_cell` | Move to the next cell on the right |
| `Shift+Tab` | `prev_cell` | Move to the previous cell on the left |
| `Enter` | `edit` | Edit the selected cell. On a group heading row, expand or collapse the group; on a checkbox cell, toggle it |
| `Backspace` | `clear` | Empty the selected cell (a pending change, saved with Ctrl+S) |
| `Delete` | `clear` | Empty the selected cell (a pending change, saved with Ctrl+S) |
| `u` | `undo` | Undo the last change |
| `Ctrl+z` | `undo` | Undo the last change |
| `Ctrl+r` | `redo` | Redo the change you undid |
| `U` | `redo` | Redo the change you undid |
| `Ctrl+Shift+z` | `redo` | Redo the change you undid |
| `a` | `new_note` | Create a new note (same as **+ New** at the right end of the view bar) |
| `c` | `new_child_note` | Create a new note linked to the selected row: a child in a table shown as a tree (the parent field is filled in), or a note in another table of the workspace whose column points to this table (mdgrid moves to that table, fills the column with the selected row, and comes back after you create or cancel). Asks which when there are two or more |
| `Ctrl+s` | `save` | Review the pending changes as a diff, then save them |
| `q` | `quit` | Quit (asks first if there are unsaved changes) |
| `Ctrl+g` | `cancel_load` | Stop loading notes |
| `?` | `help` | Show the help with every key |
| `:` | `palette` | Open the command palette (also takes a row number like `:120`, `w` to save, `q` to quit, `wq` or `x` to save and quit) |
| `Ctrl+p` | `palette` | Open the command palette |
| `x` | `action_menu` | Open the menu of actions you can use on the selected cell, each with its current key (right-clicking a cell also opens it) |
| `e` | `open_editor` | Open the selected note in your editor, and reload it when you return |
| `y` | `copy` | Copy the value of the selected cell (with rows selected: the current column's values of those rows, one per line) to the clipboard |
| `Ctrl+c` | `copy` | Copy the value of the selected cell (with rows selected: the current column's values of those rows, one per line) to the clipboard |
| `Y` | `copy_row` | Copy the whole row (or the selected rows) to the clipboard as tab-separated text with a header |
| `O` | `open_url` | Open the `http://` or `https://` URL of the selected row in the default browser (the same as `o` in the tree; the URL is shown in the message line first; when the selected cell is a URL, that URL; clicking the selected URL cell again also opens it) |
| `[` | `prev_view` | Switch to the previous view |
| `]` | `next_view` | Switch to the next view |
| `<` | `narrow_column` | Make the column narrower |
| `>` | `widen_column` | Make the column wider |
| `s` | `sort_column` | Sort by this column: ascending, descending, then off. The sort is kept for the view (the `.base` file is not changed) |
| `S` | `sort_menu` | Open the sort window (same as the **Sort** button in the view bar) |
| `\|` | `filter_menu` | Open the filter window (same as the **Filter** button in the view bar) |
| `w` | `workspace_drawer` | Open the table list (same as clicking the table name in the header breadcrumb) |
| `W` | `workspace_switch` | Open the list of favorite workspaces to switch to (same as clicking the scope name in the header breadcrumb; see `switch`) |
| `-` | `hide_column` | Hide the column |
| `+` | `show_column` | Bring back the last hidden column |
| `R` | `relation_map` | Open the relation map of the registered tables |
| `T` | `tree_view` | Open the tree view (when the table has a tree setting in the workspace marker) |
| `C` | `gantt_view` | Open the gantt screen (when the workspace marker has a gantt setting, or the date columns can be inferred) |
| `B` | `kanban_view` | Open the kanban screen (when the workspace marker has a kanban setting, or a status column can be inferred) |
| `P` | `chart_view` | Open the chart screen (on a table with columns) |
| `Z` | `toggle_tree` | Fold or unfold the child rows (when the view arranges rows by parent) |
| `A` | `add_column` | Add a column for a key no note has yet (type its name); saving writes the key only to the notes you fill in |
| `H` | `move_column_left` | Move the column to the left |
| `L` | `move_column_right` | Move the column to the right |
| `F` | `freeze_columns` | Freeze the columns up to this one so they stay visible when scrolling (press again to unfreeze) |
| `/` | `search` | Find text (highlights the matches; the rows stay) |
| `n` | `search_next` | Go to the next match |
| `N` | `search_prev` | Go to the previous match |
| `\` | `quick_filter` | Search: show only the rows containing some text (the search bar above the table) |
| `*` | `highlight_same` | Highlight the rows with the same value as the selected cell |
| `,` | `filter_same` | Show only the rows with the same value as the selected cell |
| `!` | `next_problem` | Jump to the next problem: a value that does not fit the column type or a rule (`[rules]`); the reason shows below |
| `@` | `prev_problem` | Jump to the previous problem |
| `%` | `frequency` | Show how many rows have each value of the selected column, and pick one to show only those rows |
| `1` | `condition_1` | Show only the rows that match the named condition given digit 1 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `2` | `condition_2` | Show only the rows that match the named condition given digit 2 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `3` | `condition_3` | Show only the rows that match the named condition given digit 3 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `4` | `condition_4` | Show only the rows that match the named condition given digit 4 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `5` | `condition_5` | Show only the rows that match the named condition given digit 5 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `6` | `condition_6` | Show only the rows that match the named condition given digit 6 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `7` | `condition_7` | Show only the rows that match the named condition given digit 7 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `8` | `condition_8` | Show only the rows that match the named condition given digit 8 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `m` | `notice` | Go to the notice box above the header (`[[notices]]`); does nothing when there is no box |
| `9` | `condition_9` | Show only the rows that match the named condition given digit 9 ([conditions]; digits go to the conditions with a `key`, the ledger marker\'s first, and only to those the current table can use), or undo it; for a condition with `hide = true`, show or hide again the rows it hides. Does nothing if no condition has this digit |
| `Space` | `mark_row` | Mark or unmark the row, then move down |
| `v` | `select_range` | Start selecting a range of rows; press again to mark the range |
| `Shift+Up` | `extend_up` | Extend the selection up |
| `Shift+Down` | `extend_down` | Extend the selection down |
| `Ctrl+a` | `select_all` | Select all rows |
| `Esc` | `clear_selection` | Clear the selection; if nothing is selected, clear the filter, then every condition turned on with a digit key or from the notice band (conditions with `hide` stay), then the search highlight |
| `K` | `detail` | Show every property of the selected note, and the first lines of its body |
| `o` | `view_settings` | Open the view settings (filters, sort, grouping, columns) |
| `f` | `focus_chips` | Move to the settings bar above the table |
| `Ctrl+w j` | `stack_down` | With stacked tables, move to the table below (nothing happens at the bottom) |
| `Ctrl+w k` | `stack_up` | With stacked tables, move to the table above (nothing happens at the top) |
| `Ctrl+w c` | `stack_close` | Take the selected table out of the stack. If it alone has pending changes, ask whether to save or discard them |
| `Ctrl+w +` | `stack_grow` | With stacked tables, make the selected table one row taller (the others keep their title and 3 rows) |
| `Ctrl+w -` | `stack_shrink` | With stacked tables, make the selected table one row shorter |

### vim mode

With `vim = true` under `[keys]`, the table also has these keys. Digits are never counts: `5j` moves one row down.

| Key | Action | Description |
|---|---|---|
| `d d` | `delete_note` | Move the notes of the marked rows (or the selected row) to the trash after confirming their names; while you select a range with `v`, `d` alone does it. `u` brings them back |
| `p` | `paste` | Put the value you last copied in mdgrid as a pending change: a cell copied with `y` goes into the selected cell, a row copied with `Y` into the columns of the same name in the marked rows (or the selected row) |
| `.` | `repeat` | Put the value you last set in a single cell into the same column of the selected row |

## `edit`

Editing a cell in the input box. Letter keys type text, so they have no actions.

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Confirm the value |
| `Esc` | `cancel` | Cancel the edit |
| `Tab` | `commit_next` | Confirm and move to the cell on the right |
| `Shift+Tab` | `commit_prev` | Confirm and move to the cell on the left |
| `Ctrl+r` | `revert` | Put back the value from before the edit |
| `Ctrl+u` | `clear_input` | Clear the whole input |
| `Up` | `list_up` | Previous candidate; in the date calendar, previous week |
| `Down` | `list_down` | Next candidate; in the date calendar, next week |
| `Ctrl+p` | `list_up` | Previous candidate; in the date calendar, previous week |
| `Ctrl+n` | `list_down` | Next candidate; in the date calendar, next week |
| `Left` | `cursor_left` | Move the cursor left; in the date calendar, previous day |
| `Right` | `cursor_right` | Move the cursor right; in the date calendar, next day |
| `PageUp` | `page_up` | In the date calendar, previous month |
| `PageDown` | `page_down` | In the date calendar, next month |
| `Shift+Left` | `prev_month` | In the date calendar, previous month; otherwise the same as Left |
| `Shift+Right` | `next_month` | In the date calendar, next month; otherwise the same as Right |
| `Shift+Up` | `prev_year` | In the date calendar, previous year; otherwise the same as Up |
| `Shift+Down` | `next_year` | In the date calendar, next year; otherwise the same as Down |
| `Ctrl+t` | `today` | In a date cell, set today's date |
| `Ctrl+s` | `create_note` | In the new note form, create the note now from any field |
| `Ctrl+e` | `create_note_edit` | In the new note form, create the note and open it in the editor |
| `Ctrl+o` | `time_focus` | In a datetime calendar, switch between the day and the time field (on the time field, Up/Down move 15 minutes and Shift+Up/Down one hour) |
| `Ctrl+d` | `clear` | In a date cell, empty the value and confirm |
| `Home` | `cursor_home` | Move the cursor to the start |
| `End` | `cursor_end` | Move the cursor to the end |
| `Backspace` | `delete_back` | Delete the character before the cursor |
| `Delete` | `delete_forward` | Delete the character after the cursor |

## `palette`

The command palette (`:` or Ctrl+P). Type to search commands by name.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Run the selected command |
| `Esc` | `close` | Close the palette |
| `Down` | `down` | Next candidate |
| `Ctrl+n` | `down` | Next candidate |
| `Up` | `up` | Previous candidate |
| `Ctrl+p` | `up` | Previous candidate |
| `Backspace` | `delete_back` | Delete the character before the cursor |

## `review`

The save review, which shows the pending changes as a diff before writing.

| Key | Action | Description |
|---|---|---|
| `Enter` | `save_all` | Save all the changes |
| `Esc` | `back` | Go back to the table without saving |
| `q` | `back` | Go back to the table without saving |
| `j` | `next_file` | Next file |
| `Down` | `next_file` | Next file |
| `k` | `prev_file` | Previous file |
| `Up` | `prev_file` | Previous file |
| `o` | `overwrite` | Write this file even though it was changed outside mdgrid |
| `d` | `discard_row` | Discard the pending changes to this file |
| `?` | `help` | Show the help |

## `detail`

The detail view, which lists every property of one note and, under a `Body` heading, the first 20 lines of its body (read-only; no heading when the note has no body). Clicking a property's line selects that property.

| Key | Action | Description |
|---|---|---|
| `Enter` | `edit` | Edit the selected property |
| `Esc` | `close` | Close the detail view |
| `K` | `close` | Close the detail view |
| `O` | `open_url` | Open the `http://` or `https://` URL of the selected property, or else the URL of the row, in the default browser (the same safeguards as `O` in the table; clicking the selected URL value again also opens it) |
| `j` | `down` | Move down |
| `Down` | `down` | Move down |
| `k` | `up` | Move up |
| `Up` | `up` | Move up |
| `PageDown` | `page_down` | Scroll down one screen |
| `PageUp` | `page_up` | Scroll up one screen |
| `g g` | `top` | Go to the top |
| `G` | `bottom` | Go to the bottom |
| `?` | `help` | Show the help |

## `quit`

The prompt shown when you quit with unsaved changes.

| Key | Action | Description |
|---|---|---|
| `s` | `quit_save` | Review and save the changes, then quit |
| `d` | `quit_discard` | Discard the changes and quit |
| `Esc` | `back` | Go back to the table |
| `?` | `help` | Show the help |

## `help`

The help screen (`?`). Select a line of the first section, the keys you can press now, with ↑↓ or a click and press `Enter`: the help closes and that action runs, the same as pressing its key on the screen you came from. Clicking the selected line again does the same. A line that groups digit keys (`1-9`) runs the first key (`1`). Lines further down (every key, the meaning of the marks) can be selected, but `Enter` does nothing there. The mouse wheel scrolls the help without moving the selection.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Run the selected key you can press now and close the help |
| `Esc` | `close` | Close the help without doing anything |
| `q` | `close` | Close the help without doing anything |
| `?` | `close` | Close the help without doing anything |
| `j` | `down` | Next item |
| `Down` | `down` | Next item |
| `k` | `up` | Previous item |
| `Up` | `up` | Previous item |
| `PageDown` | `page_down` | Down one screen |
| `PageUp` | `page_up` | Up one screen |
| `g g` | `top` | First item |
| `G` | `bottom` | Last item |

## `search`

Typing a term to find (`/`). The rows stay; matches are highlighted and `n` and `N` move between them.

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Find the term |
| `Esc` | `cancel` | Cancel finding |
| `Backspace` | `delete_back` | Delete the character before the cursor |

## `filter`

Typing a search term (`\`, the search bar above the table). Every keystroke shows only the rows containing the term.

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Leave the bar, keeping the search |
| `Esc` | `cancel` | Clear the search |
| `Backspace` | `delete_back` | Delete the character before the cursor |

## `settings`

The view settings screen (`o`, or the **⚙ Settings** button at the top right), where you change filters, sorting, grouping, and columns. The sections are listed on the left with numbers, and the selected section's items are on the right. Keys work on one side at a time, the one with the `>` mark: `h` moves to the list on the left, `l` or `Enter` back to the items.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Choose the selected item (in the list on the left: go to the section's items) |
| `Space` | `toggle` | Turn the selected item on or off |
| `Esc` | `cancel` | Go back, or cancel |
| `Tab` | `next_section` | Move to the next section |
| `Shift+Tab` | `prev_section` | Move to the previous section |
| `1` | `go_to_section_1` | Go to section 1 of the list on the left |
| `2` | `go_to_section_2` | Go to section 2 of the list on the left |
| `3` | `go_to_section_3` | Go to section 3 of the list on the left |
| `4` | `go_to_section_4` | Go to section 4 of the list on the left |
| `5` | `go_to_section_5` | Go to section 5 of the list on the left |
| `6` | `go_to_section_6` | Go to section 6 of the list on the left |
| `7` | `go_to_section_7` | Go to section 7 of the list on the left |
| `8` | `go_to_section_8` | Go to section 8 of the list on the left |
| `9` | `go_to_section_9` | Go to section 9 of the list on the left |
| `j` | `down` | Move down (in the list on the left: the next section) |
| `Down` | `down` | Move down (in the list on the left: the next section) |
| `k` | `up` | Move up (in the list on the left: the previous section) |
| `Up` | `up` | Move up (in the list on the left: the previous section) |
| `h` | `left` | Move to the list of sections on the left (between buttons: move left) |
| `Left` | `left` | Move to the list of sections on the left (between buttons: move left) |
| `l` | `right` | From the list on the left, go to the section's items (between buttons: move right) |
| `Right` | `right` | From the list on the left, go to the section's items (between buttons: move right) |
| `K` | `move_item_up` | Move the item up |
| `Shift+Up` | `move_item_up` | Move the item up |
| `J` | `move_item_down` | Move the item down |
| `Shift+Down` | `move_item_down` | Move the item down |
| `d` | `remove_item` | Remove the item |
| `Delete` | `remove_item` | Remove the item |
| `Backspace` | `remove_item` | Remove the item |
| `?` | `help` | Show the help |

## `settings_input`

Typing a value in the view settings (such as the text for "contains" or a value to compare with).

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Confirm the value |
| `Esc` | `cancel` | Go back |
| `Backspace` | `delete_back` | Delete the character before the cursor |

## `chips`

The settings bar above the table (`f`), which shows the view's filters and sorting as items.

| Key | Action | Description |
|---|---|---|
| `Backspace` | `remove_item` | Remove the selected item from the view |
| `Delete` | `remove_item` | Remove the selected item from the view |
| `h` | `left` | Select the item on the left |
| `Left` | `left` | Select the item on the left |
| `l` | `right` | Select the item on the right |
| `Right` | `right` | Select the item on the right |
| `Enter` | `view_settings` | Open the view settings |
| `Esc` | `close` | Go back to the table |
| `f` | `close` | Go back to the table |
| `?` | `help` | Show the help |

## `notice`

The notice box above the header (`m`, or a click on it), with the items written in `[[notices]]`.

| Key | Action | Description |
|---|---|---|
| `h` | `left` | Select the item on the left |
| `Left` | `left` | Select the item on the left |
| `l` | `right` | Select the item on the right |
| `Right` | `right` | Select the item on the right |
| `k` | `up` | Select the item on the left |
| `Up` | `up` | Select the item on the left |
| `j` | `down` | Select the item on the right |
| `Down` | `down` | Select the item on the right |
| `Enter` | `run` | Go to the item: narrow the table by the condition, list the tables with matching rows, or open the note in the editor |
| `Esc` | `close` | Go back to the table |
| `m` | `close` | Go back to the table |
| `?` | `help` | Show the help |

## `list_select`

Picking values for a list property. Enter checks or unchecks the highlighted value and leaves the window open, so you can check several in a row. When you are done, press Tab, or Enter on the **[ OK ]** line at the bottom. Letter keys type into the search box. The window's main keys are also shown on its bottom edge, and you can click them.

| Key | Action | Description |
|---|---|---|
| `Tab` | `commit` | Confirm the checks and close, from any line |
| `Enter` | `run` | Check or uncheck the highlighted value (the window stays open); on **[ OK ]**, confirm. While searching, add the highlighted candidate and clear the search |
| `Space` | `toggle` | Check or uncheck the highlighted value; while searching, type a space |
| `Esc` | `cancel` | Close without changing anything |
| `Ctrl+r` | `revert` | Put back the selection from when the list was opened |
| `Ctrl+s` | `create_note` | In the new note form, create the note now from any field |
| `Ctrl+e` | `create_note_edit` | In the new note form, create the note and open it in the editor |
| `Shift+Tab` | `commit_prev` | In the new note form, go back to the previous field |
| `Up` | `list_up` | Previous candidate |
| `Down` | `list_down` | Next candidate |
| `Ctrl+p` | `list_up` | Previous candidate |
| `Ctrl+n` | `list_down` | Next candidate |
| `PageUp` | `page_up` | Scroll up one screen |
| `PageDown` | `page_down` | Scroll down one screen |
| `Backspace` | `delete_back` | Delete the character before the cursor |

Clicking a value checks or unchecks it, and clicking **[ OK ]** confirms. A list field in the new note form uses the same keys: Enter checks a value, and Tab or **[ OK ]** confirms the field and moves on (on the last field, it creates the note). There, Esc closes only this window and keeps the form. After that, Enter opens it again, Tab moves to the next field, and a second Esc closes the form.

## `menu`

The menu of actions for the selected cell (`x` or a right-click on a cell). It groups the actions into Cell, Column, Row, and View and file, and shows each action's current key. Actions you can't use there (such as editing a read-only cell) are left out. Pressing a key shown next to an item runs that action, just like pressing it in the `table` mode.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Run the selected action and close the menu |
| `Esc` | `close` | Close the menu without doing anything |
| `j` | `down` | Next item |
| `Down` | `down` | Next item |
| `k` | `up` | Previous item |
| `Up` | `up` | Previous item |
| `g g` | `top` | Go to the first item |
| `G` | `bottom` | Go to the last item |

Clicking an item runs it; clicking outside the menu just closes it. If the terminal is too small to show the menu, it doesn't open and a message explains why. After a key-sequence prefix such as `g`, `Esc` only cancels the prefix.

## `freq`

The value counts of the selected column (`%`). Each value is listed with its number of rows and its share of all the rows shown now, most common first (values with the same count in value order). A list property is counted per item, so the shares can add up to more than 100%, and rows without a value are counted as `(empty)`. Only the rows left after the current filters are counted, including a same-value filter from `,` or from an earlier value count: `Enter` adds to that filter instead of replacing it, so the count next to a value is exactly the number of rows you get. It doesn't open while notes are still loading, and it closes if the table is rebuilt (for example, when a note changes outside mdgrid).

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Show only the rows with the selected value (like `,`) and close |
| `Esc` | `close` | Close without doing anything |
| `j` | `down` | Next value |
| `Down` | `down` | Next value |
| `k` | `up` | Previous value |
| `Up` | `up` | Previous value |
| `g g` | `top` | Go to the first value |
| `G` | `bottom` | Go to the last value |

Clicking a value shows only its rows; clicking outside just closes the window. If the terminal is too small to show it, it doesn't open and a message explains why. `Esc` in the `table` mode clears the filter again.

## `sorts`

The sort window (`S`, or the **Sort** button in the view bar; it opens right below the button). It lists the sort rules of the current view (column and direction) and a last line to add one. Every change applies to the table right away and is kept for the view, like a sort from a column heading.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Flip the direction of the selected rule, or add a rule (then pick a column) |
| `Esc` | `close` | Close the window (while picking a column, go back to the rules) |
| `q` | `close` | Close the window |
| `j` | `down` | Next line |
| `Down` | `down` | Next line |
| `k` | `up` | Previous line |
| `Up` | `up` | Previous line |
| `d` | `remove_item` | Remove the selected rule |
| `Delete` | `remove_item` | Remove the selected rule |
| `Backspace` | `remove_item` | Remove the selected rule |
| `K` | `move_item_up` | Move the rule up (it sorts first) |
| `Shift+Up` | `move_item_up` | Move the rule up |
| `J` | `move_item_down` | Move the rule down |
| `Shift+Down` | `move_item_down` | Move the rule down |

Clicking a line does what `Enter` does, and clicking the `×` at the right end of a rule removes it; clicking outside closes the window.

## `drawer`

The table list (`w`, `:workspace_drawer`, or the table name in the header breadcrumb). It switches within the workspace: it slides over the table from the left and lists only the pages and tables of the current scope (the workspace, or the registered tables when there is none), marking the current table with `●`. Other workspaces, **Where you started** and favorites are not listed here; use the switch list (`switch`) for them. Selecting a table switches to it (with unsaved changes, you are asked to save or discard first). While a page is open, only the page name has the `●`, and selecting a table closes the page and opens that table on its own. It is hidden until opened.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Switch to the selected table or page |
| `Esc` | `close` | Close the list |
| `q` | `close` | Close the list |
| `w` | `close` | Close the list |
| `j` | `down` | Next line |
| `Down` | `down` | Next line |
| `k` | `up` | Previous line |
| `Up` | `up` | Previous line |
| `g g` | `top` | First line |
| `G` | `bottom` | Last line |

Clicking a line does what `Enter` does; clicking outside the list (including the table name in the header breadcrumb) closes it. Clicking the scope name in the header breadcrumb closes it and opens the switch list.

## `filter_window`

The filter window (`|`, or the **Filter** button in the view bar; it opens right below the button). It lists the filter conditions of the current view as a table, one condition per line (column, comparison, value, ×), and a last line to add one. On a condition line, ←→ picks a field and Enter edits that field in place. To add, pick a column, then a kind (pick from the values, contains, does not contain, compare, is empty, is not empty), then the value. Every change applies to the table right away and is kept for the view (the same filters as in the view settings).

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Edit the condition, or add one (pick a column, then a kind); inside a step, go on with the selected item. In the value list, check or uncheck the value (on **[ OK ]** at the bottom, confirm) |
| `Esc` | `close` | Close the window (while adding, go back to the conditions) |
| `q` | `close` | Close the window |
| `j` | `down` | Next line |
| `Down` | `down` | Next line |
| `k` | `up` | Previous line |
| `Up` | `up` | Previous line |
| `Space` | `toggle` | Check or uncheck a value in the value list (on the first line, switch keep / hide) |
| `Tab` | `commit` | In the value list, confirm the checks, from any line |
| `d` | `remove_item` | Remove the selected condition |
| `Delete` | `remove_item` | Remove the selected condition |
| `Backspace` | `remove_item` | Remove the selected condition |
| `h` | `left` | Previous field of the condition (column / comparison / value) |
| `Left` | `left` | Previous field of the condition |
| `l` | `right` | Next field of the condition |
| `Right` | `right` | Next field of the condition |

Clicking a line does what `Enter` does (in the value list, it checks the value, and **[ OK ]** confirms); clicking the `×` at the right of a condition removes it. The keys of the current step are on the bottom edge of the window and can be clicked. Clicking outside closes the window.

## `filter_input`

Typing the value for contains, does not contain, or compare in the filter window.

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Add the condition with this value (if the value doesn't fit the column type, say why) |
| `Esc` | `cancel` | Go back to the conditions without adding |
| `Backspace` | `delete_back` | Delete the character before the cursor |


## `relations`

The relation map (`R`, `:relation_map`, or **Relation map** in the screen type menu at the top right, which lists it once you have registered tables). It shows the registered tables (and the current one) as boxes with their row counts and columns, and draws an arrow from each column that links to another table: `N` on the linking side, `1` (many-to-one) or `N` (many-to-many, a list of links) on the target side. The panes follow the terminal size: 120 columns or more show the tables, the map and the details side by side; 80 or more show the map and the details; narrower terminals show the tables and links as a list. With 30 rows or more, the linked records of the selected link appear below.

The mouse works too: click a table (in the list or its box on the map) to select it, and click the selected table again to open it. Click a line, an arrow or a label (or a link in the narrow list) to select that link. Click a linked record to open its note. The wheel moves between tables.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Open the selected table |
| `Esc` | `close` | Back to the table |
| `R` | `relation_map` | Back to the table |
| `q` | `close` | Back to the table |
| `j` | `down` | Next table |
| `Down` | `down` | Next table |
| `k` | `up` | Previous table |
| `Up` | `up` | Previous table |
| `l` | `right` | Next link of the selected table |
| `Right` | `right` | Next link of the selected table |
| `h` | `left` | Previous link of the selected table |
| `Left` | `left` | Previous link of the selected table |
| `a` | `new_note` | Open the selected table and start a new note (same as **+ New** at the right end of the second line) |
| `W` | `workspace_switch` | Open the list of favorite workspaces to switch to (see `switch`) |
| `:` | `palette` | The command palette (closing it returns to the relation map) |
| `?` | `help` | Show help |

Choosing **Table** in the screen type menu at the top right (**◇ Relation map ▾**) goes back to the table. The **⚙ Settings** button at the top right goes back to the table and opens the view settings.

## `screen_menu`

The screen type menu (the **▦ Table ▾** button at the top right; on the relation map it reads **◇ Relation map ▾**). It opens right below the button and lists the screen types: Table, Relation map when there are linked tables, Tree when the workspace marker gives the table a tree setting, Gantt and Kanban when their columns are set in the marker or can be inferred, and Chart when the table has columns. The current type is marked with `●`, and a type that has a key shows it at the right (`R` for the relation map, `T` for the tree, `C` for the gantt, `B` for the kanban, `P` for the chart). Selecting a type switches to it. `R`, `T`, `C`, `B` and `P` switch without opening the menu.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Switch to the selected screen |
| `Esc` | `close` | Close the menu |
| `q` | `close` | Close the menu |
| `j` | `down` | Next line |
| `Down` | `down` | Next line |
| `k` | `up` | Previous line |
| `Up` | `up` | Previous line |

Clicking a line does what `Enter` does; clicking outside the menu (including the button) closes it.

## `column_type`

The column type menu (CE-35): click the type mark of a column heading, or run `column_type` from the palette or the action menu. It opens right below the column heading and lists **Auto** (with the type guessed from the values) and the types Text, Number, Checkbox, Date, Date and time and List; the current one is marked with `●`. The chosen type is written to the table in `views.toml` (`[types]`) and applies at once; notes are not changed. Not for `file.*` and `formula.*` columns.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Use the selected type |
| `Esc` | `close` | Close the menu |
| `q` | `close` | Close the menu |
| `j` | `down` | Next line |
| `Down` | `down` | Next line |
| `k` | `up` | Previous line |
| `Up` | `up` | Previous line |

Clicking a line does what `Enter` does; clicking outside the menu closes it.

## `tree`

The tree view (`T`, `:tree_view`, or **Tree** in the screen type menu at the top right). It is offered only for a table whose `[[table]]` in the workspace marker has `tree = { path = "<column>" }` (see [Configuration](config.md)). The value of that column (such as `bar > dev > Web`) is split at the separator; the folders are on the left and the rows directly in the selected folder on the right. Each folder shows how many rows are under it (including its subfolders). Rows with an empty value go to **(none)**. The rows on the right are sorted by the `order` column (smallest first; values that are not numbers and empty values last; equal values by note name) and show the columns of the current view. The tree is built from the rows that pass the current view. The panes follow the terminal width: side by side when wide, one at a time when narrow (switch with the tabs on top or `Tab`).

The mouse works too: click a folder to select it, click its `▾` or `▸` mark to close or open it, click a row on the right to select it, and click the selected row again to go to that row in the table. The wheel moves the selection up and down.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Open or close the folder / show the row in the table |
| `Esc` | `close` | Back to the table |
| `T` | `tree_view` | Back to the table |
| `q` | `close` | Back to the table |
| `j` | `down` | Next folder or row |
| `Down` | `down` | Next folder or row |
| `k` | `up` | Previous folder or row |
| `Up` | `up` | Previous folder or row |
| `l` | `right` | Open the folder / to the rows |
| `Right` | `right` | Open the folder / to the rows |
| `h` | `left` | Close the folder / to the folders |
| `Left` | `left` | Close the folder / to the folders |
| `Space` | `toggle` | Open or close the folder |
| `Tab` | `next_section` | Switch between the folders and the rows |
| `Shift+Tab` | `prev_section` | Switch between the folders and the rows |
| `o` | `open_url` | Open the `http://` or `https://` URL of the selected row in the default browser (the URL is shown in the message line first; other values, URLs that point to an executable file and URLs with characters that change the text direction are never opened, with the reason shown; clicking the URL value of the selected row again also opens it) |
| `x` | `action_menu` | Actions on the selected row (open the URL, show the row in the table) |
| `W` | `workspace_switch` | Open the list of favorite workspaces to switch to (see `switch`) |
| `:` | `palette` | Command palette (closing it returns to the tree) |
| `?` | `help` | Show the help |

## `gantt`

The gantt screen (`C`, `:gantt_view`, or **Gantt** in the screen type menu at the top right). It is offered on a table whose `[[table]]` in the workspace marker has `gantt = { start = "<column>", end = "<column>" }` (`days = "<column>"` instead of `end` also works), or, without a setting, on a table whose date columns can be inferred by name (start: start, begin, from, 開始; end: end, due, finish, until, to, 期限, 終了). See [Configuration](config.md). The rows of the current view are listed with their names on the left and a time axis with bars on the right. The unit is a day, a week or a month; on opening, the finest unit that fits the range of the rows in the width. Today's column is marked (`┊` in the rows, and `▾` in the header, or the header digit shown reversed when today falls on it). In the month unit a month takes three characters, one for each third of the month. With a color theme (not `default`), the background of the time axis alternates by month (by year in the month unit), and weekends get a slightly stronger tint in the day unit. A row with only one of the two dates shows `◆`; a row with neither shows no bar. With parent/child rows in the view settings, the order, indent and folding are the same as in the table, and so are the WBS numbers and progress (`█` and `░` in the bar); a parent without dates gets the span of its descendants. The control bar on the bottom edge of the window (`◀ ▶`, the units, Today, Week, Month, Quarter, All) can be clicked too. `H` and `L` move the selected row's bar by one unit, and `<` and `>` shorten or extend its end (the dates are written as pending changes; `u` undoes, `Ctrl+S` goes to the save confirmation). To type a date, edit it in the table.

The mouse works too: click a row to select it, click the selected row again to move to that row in the table. The wheel selects up and down; the horizontal wheel or shift+wheel moves the time axis.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Show the row in the table |
| `Esc` | `close` | Back to the table |
| `C` | `gantt_view` | Back to the table |
| `q` | `close` | Back to the table |
| `j` | `down` | Next row |
| `Down` | `down` | Next row |
| `k` | `up` | Previous row |
| `Up` | `up` | Previous row |
| `l` | `right` | Scroll the time axis later |
| `Right` | `right` | Scroll the time axis later |
| `h` | `left` | Scroll the time axis earlier |
| `Left` | `left` | Scroll the time axis earlier |
| `+` | `zoom_in` | Finer unit (month → week → day) |
| `-` | `zoom_out` | Coarser unit (day → week → month) |
| `t` | `today` | Go to today |
| `Z` | `toggle_tree` | Fold or unfold the child rows of the selected row |
| `Space` | `toggle_tree` | Fold or unfold the child rows of the selected row |
| `W` | `workspace_switch` | Open the list of favorite workspaces to switch to (see `switch`) |
| `:` | `palette` | The command palette (closing it returns to the gantt) |
| `?` | `help` | Show the help |
| `r` | `gantt_range` | Switch the period (this week → this month → quarter → all) |
| `L` | `bar_later` | Move the bar one unit later (rewrites the dates) |
| `H` | `bar_earlier` | Move the bar one unit earlier (rewrites the dates) |
| `>` | `bar_longer` | Extend the end of the bar by one unit |
| `<` | `bar_shorter` | Shorten the end of the bar by one unit |
| `u` | `undo` | Undo |
| `Ctrl+z` | `undo` | Undo |
| `U` | `redo` | Redo |
| `Ctrl+r` | `redo` | Redo |
| `Ctrl+s` | `save` | Save |

## `kanban`

The kanban screen (`B`, `:kanban_view`, or **Kanban** in the screen type menu at the top right). It is offered on a table whose `[[table]]` in the workspace marker has `kanban = { column = "<column>" }`, or, without a setting, on a table with a column named status, state, stage, 状態, ステータス or 段階 (any case; a list column is never picked). See [Configuration](config.md). The rows of the current view become cards in lanes, one lane per value of the column. The lanes follow the allowed values of the column (`[rules.<column>] values`), then any other value in the order the rows show it, and last **(none)** for rows without a value; a value with no rows still gets an empty lane. Each lane header shows the value and the number of cards, and each card shows the row's name. When the lanes don't fit the width, the screen slides to keep the selected lane visible and marks the hidden side with `‹` or `›`. The selected card has `>`, and a card with pending changes has `*`.

`H` and `L` move the selected card to the lane on the left or right: the row's value in the column becomes that lane's value, as a pending change like an edit in the table (`*`, and the unsaved count goes up). Nothing is written until you save; `Ctrl+s` goes back to the table and opens the review of the pending changes. `u` undoes a move. Cards are not moved into **(none)** (clear the value in the table), and nothing moves when mdgrid was opened with `--readonly` or the cell is read-only; the message line says why.

The mouse works too: click a card to select it, click the selected card again to move to that row in the table, and click a lane header to select the lane. The wheel selects cards up and down; the horizontal wheel or shift+wheel moves between lanes.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Show the card's row in the table |
| `Esc` | `close` | Back to the table |
| `B` | `kanban_view` | Back to the table |
| `q` | `close` | Back to the table |
| `j` | `down` | Next card |
| `Down` | `down` | Next card |
| `k` | `up` | Previous card |
| `Up` | `up` | Previous card |
| `l` | `right` | Lane to the right |
| `Right` | `right` | Lane to the right |
| `h` | `left` | Lane to the left |
| `Left` | `left` | Lane to the left |
| `L` | `move_card_right` | Move the card to the lane on the right (rewrites the status) |
| `H` | `move_card_left` | Move the card to the lane on the left (rewrites the status) |
| `u` | `undo` | Undo the last change |
| `Ctrl+z` | `undo` | Undo the last change |
| `U` | `redo` | Redo the change you undid |
| `Ctrl+r` | `redo` | Redo the change you undid |
| `Ctrl+s` | `save` | Back to the table and review the pending changes before saving |
| `W` | `workspace_switch` | Open the list of favorite workspaces to switch to (see `switch`) |
| `:` | `palette` | The command palette (closing it returns to the kanban) |
| `?` | `help` | Show the help |

## `chart`

The chart screen (`P`, `:chart_view`, or **Chart** in the screen type menu at the top right). It is offered on any table with columns. It counts the rows of the current view by the values of a column, or totals a number column, and shows them as a bar, stacked bar, line, scatter or heatmap chart. With `chart = { kind = "bar", x = "status" }` (an array for several charts) in the table's `[[table]]` of the workspace marker, the chart opens that way (see [Configuration](config.md)). Without a setting it opens as bars, splitting by the first of: a status column (status, state and so on), a column with allowed values (`[rules.<column>] values`), a checkbox column, a list column, a date column, the first column; the value is the count. **(inferred)** at the end of the title marks a guessed column.

How rows split depends on the column type. Text and checkbox columns split by value, list columns by element (a row can count in several items), and rows without a value go to **(none)**. Date columns split into days, weeks or months, with empty ranges shown as 0. Number columns split into equal ranges (up to 10), which makes a histogram. Rows whose date or number cannot be read, and rows whose summed value is not a number, are left out, and the line under the title says **N left out**.

- **Bar**: one horizontal bar per item; the largest value fills the bar.
- **Stacked bar**: bars split by the values of a breakdown column (the top five and **other**). `%` turns each bar into percentages. Fill characters (`█` `▓` `▒` `░` `▞`) tell the parts apart without color.
- **Line**: joins the value of each date or number range. `c` makes it cumulative. With a breakdown column there is one line per value (up to five), told apart by point characters (`*` `+` `o` `x` `#`).
- **Scatter**: one point per row, with a number or date column across and another one up. `j` and `k` pick a point; `Enter` goes to its row in the table.
- **Heatmap**: the value for each pair of the split column (down) and the breakdown column (across), as a number and a shade (` ` `░` `▒` `▓` `█`).

On the screen, `Tab` changes the kind, `x` the column to split by, `y` the value (count or the sum of a number column), `a` the aggregate, `b` the breakdown column, and `+` `-` the date unit. These changes stay on the screen; the marker is not rewritten. When a kind has no fitting column, it is skipped with a reason on the message line. With several charts in the marker, `[` and `]` move between them. `Enter` on a bar, stacked bar or heatmap cell narrows the table to the rows with that value and goes back to it (the same filter as `Enter` in the `%` frequency table; date and number ranges do not narrow). Values are not edited here.

The mouse works too: click an item to select it, and click the selected item again for the same as `Enter`. The wheel moves the selection.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Narrow the table to this value (a scatter goes to the point's row in the table) |
| `Esc` | `close` | Back to the table |
| `P` | `chart_view` | Back to the table |
| `q` | `close` | Back to the table |
| `j` | `down` | Next item |
| `Down` | `down` | Next item |
| `k` | `up` | Previous item |
| `Up` | `up` | Previous item |
| `l` | `right` | Item to the right (a line's range, a heatmap column) |
| `Right` | `right` | Item to the right |
| `h` | `left` | Item to the left |
| `Left` | `left` | Item to the left |
| `Tab` | `chart_kind_next` | Next kind (bar → stacked → line → scatter → heatmap) |
| `Shift+Tab` | `chart_kind_prev` | Previous kind |
| `x` | `chart_x` | Next column to split by |
| `y` | `chart_y` | Next value (count → number columns) |
| `a` | `chart_agg` | Next aggregate (sum → average → min → max) |
| `b` | `chart_by` | Next breakdown column |
| `+` | `chart_finer` | Finer date unit (month → week → day) |
| `-` | `chart_coarser` | Coarser date unit (day → week → month) |
| `[` | `chart_prev_saved` | Previous chart in the marker |
| `]` | `chart_next_saved` | Next chart in the marker |
| `c` | `chart_cumulative` | Toggle cumulative (line) |
| `%` | `chart_percent` | Toggle percentages (stacked bar) |
| `W` | `workspace_switch` | Open the list of favorite workspaces to switch to (see `switch`) |
| `:` | `palette` | The command palette (closing it returns to the chart) |
| `?` | `help` | Show help |

## `switch`

The list for switching workspaces (`W` on the table, relation map, tree, gantt, kanban and chart screens, or `:workspace_switch`). It opens over the current screen at the left, below the header, and lists your favorite workspaces with numbers in the order you added them (favorites are kept in `favorites` at the top of `workspaces.toml`; see below for adding them). The workspace you are in is marked with `●`. When the workspace you are in is not a favorite, the list ends with **+ Add <name> to favorites**; that line or `a` adds it and keeps the list open. `d` removes the selected favorite, and `K` and `J` move it up or down (its number moves with it). `:workspace_favorite` in the palette also adds or removes the current workspace. When you have moved away from where you started (a plain folder, a `.base`, a detected root folder, or a workspace), the list starts with **Where you started**, numbered `0`. Choosing a workspace opens the table and view you last looked at there in this run, or, the first time, the table named in the marker's `open` or the first table. **Where you started** opens the table you started with, in the view you last looked at there. With unsaved changes, you are asked to save or discard first.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Go to the selected workspace or to where you started |
| `Esc` | `close` | Close the list and stay where you are |
| `q` | `close` | Close the list and stay where you are |
| `j` | `down` | Next line |
| `Down` | `down` | Next line |
| `k` | `up` | Previous line |
| `Up` | `up` | Previous line |
| `a` | `workspace_favorite` | Add the workspace you are in to favorites (if it already is one, only says so) |
| `d` | `remove_item` | Remove the selected favorite |
| `Delete` | `remove_item` | Remove the selected favorite |
| `Backspace` | `remove_item` | Remove the selected favorite |
| `K` | `move_item_up` | Move the selected favorite up one place |
| `Shift+Up` | `move_item_up` | Move the selected favorite up one place |
| `J` | `move_item_down` | Move the selected favorite down one place |
| `Shift+Down` | `move_item_down` | Move the selected favorite down one place |
| `0` | `choose_0` | Go back to where you started |
| `1` | `choose_1` | Go to favorite 1 |
| `2` | `choose_2` | Go to favorite 2 |
| `3` | `choose_3` | Go to favorite 3 |
| `4` | `choose_4` | Go to favorite 4 |
| `5` | `choose_5` | Go to favorite 5 |
| `6` | `choose_6` | Go to favorite 6 |
| `7` | `choose_7` | Go to favorite 7 |
| `8` | `choose_8` | Go to favorite 8 |
| `9` | `choose_9` | Go to favorite 9 |

While the list is open, the bottom band says that the digit keys (`0`-`9`) go to the numbered place, and shows `a` and `d` (the `a` and `d` on the bottom edge of the list can be clicked too). Pressing the number of the place you are in closes the list and tells you that you are already in that workspace. The key that opened the list (`W`) also closes it. Clicking a line does what `Enter` does; clicking outside the list closes it (clicking the table name in the header breadcrumb closes it and opens the drawer). Clicking the scope name in the header breadcrumb, or the **W ⇄** button at the right of the header (shown when you have favorites or have moved away from where you started; on a narrow terminal it is dropped before the settings button) opens it too.

## `input_normal`

Normal mode of an input box in vim mode (`vim = true` under `[keys]`). Every input box (cell, search, palette, settings value, new note name) starts in insert mode, where you type as usual; Esc switches to normal mode, and Esc again cancels. Enter confirms in both modes. The bottom band shows `-- INSERT --` or `-- NORMAL --`, and the cursor turns into a bar or a block. Words break where vim breaks them, also between kanji, hiragana and katakana. Keys not in this table (Enter, Esc, arrows, Ctrl keys) work as in the input box's own mode; other letters do nothing.

| Key | Action | Description |
|---|---|---|
| `h` | `cursor_left` | Move left |
| `l` | `cursor_right` | Move right |
| `0` | `cursor_home` | Move to the start |
| `$` | `cursor_end` | Move to the last character |
| `w` | `word_next` | Move to the start of the next word |
| `b` | `word_prev` | Move to the start of the word, or of the previous word |
| `e` | `word_end` | Move to the end of the word |
| `i` | `insert` | Insert before the cursor |
| `a` | `append` | Insert after the cursor |
| `I` | `insert_start` | Insert at the start |
| `A` | `append_end` | Insert at the end |
| `x` | `delete_forward` | Delete the character under the cursor |
| `X` | `delete_back` | Delete the character before the cursor |
| `d w` | `delete_word` | Delete to the start of the next word |
| `d b` | `delete_word_back` | Delete back to the start of the word |
| `d $` | `delete_to_end` | Delete to the end |
| `D` | `delete_to_end` | Delete to the end |
| `d d` | `clear_input` | Delete the whole text |
| `c w` | `change_word` | Delete to the end of the word and insert |
| `c $` | `change_to_end` | Delete to the end and insert |
| `C` | `change_to_end` | Delete to the end and insert |
| `c c` | `change_line` | Delete the whole text and insert |
| `u` | `undo` | Undo the last change in the input box |

## Actions

Every action name you can write in the keys section of the config file. Each one works only in the modes where it appears in the tables above.

- `up` — move up (previous candidate in the palette)
- `down` — move down (next candidate in the palette)
- `left` — move left
- `right` — move right
- `first_column` — move to the first column
- `last_column` — move to the last column
- `page_up` — scroll up one screen (previous month in the date calendar)
- `page_down` — scroll down one screen (next month in the date calendar)
- `half_page_up` — scroll up half a screen
- `half_page_down` — scroll down half a screen
- `top` — go to the first row or the top
- `bottom` — go to the last row or the bottom
- `next_cell` — move to the next cell on the right
- `prev_cell` — move to the previous cell on the left
- `edit` — edit the selected cell or property
- `clear` — empty the selected cell
- `undo` — undo the last change
- `redo` — redo the change you undid
- `new_note` — create a new note
- `new_child_note` — create a note linked to the selected row (a child, or a note in another table that points to it)
- `save` — review and save the pending changes
- `quit` — quit mdgrid
- `cancel_load` — stop loading notes
- `help` — show the help
- `palette` — open the command palette
- `action_menu` — open the menu of actions for the selected cell
- `open_editor` — open the note in your editor
- `copy` — copy the cell's value (with rows selected, the current column's values)
- `copy_row` — copy the row or the selected rows
- `prev_view` — switch to the previous view
- `next_view` — switch to the next view
- `narrow_column` — make the column narrower
- `widen_column` — make the column wider
- `sort_column` — sort by the column (kept for the view)
- `hide_column` — hide the column
- `show_column` — bring back the last hidden column
- `relation_map` — switch between the table and the relation map
- `toggle_tree` — fold or unfold the child rows of the selected row
- `tree_view` — switch between the table and the tree view
- `gantt_view` — switch between the table and the gantt screen
- `zoom_in` — finer unit of the gantt time axis (month → week → day)
- `zoom_out` — coarser unit of the gantt time axis (day → week → month)
- `gantt_range` — switch the gantt period (this week → this month → quarter → all)
- `bar_earlier` — move the selected gantt bar one unit earlier (rewrites the dates)
- `bar_later` — move the selected gantt bar one unit later (rewrites the dates)
- `bar_longer` — extend the end of the selected gantt bar by one unit
- `bar_shorter` — shorten the end of the selected gantt bar by one unit
- `kanban_view` — switch between the table and the kanban screen
- `move_card_left` — move the kanban card to the lane on the left (rewrites the status)
- `move_card_right` — move the kanban card to the lane on the right (rewrites the status)
- `chart_view` — switch between the table and the chart screen
- `chart_kind_next` — next chart kind (bar → stacked → line → scatter → heatmap)
- `chart_kind_prev` — previous chart kind
- `chart_x` — next column to split the chart by
- `chart_y` — next chart value (count → number columns)
- `chart_agg` — next chart aggregate (sum → average → min → max)
- `chart_by` — next breakdown column of the chart
- `chart_finer` — finer date unit of the chart (month → week → day)
- `chart_coarser` — coarser date unit of the chart (day → week → month)
- `chart_prev_saved` — previous chart in the marker
- `chart_next_saved` — next chart in the marker
- `chart_cumulative` — toggle a cumulative line
- `chart_percent` — toggle percentages of a stacked bar
- `open_url` — open the http or https URL of the selected row in the browser (`O` in the table, `o` in the tree; also from the palette and the action menu)
- `add_column` — add a column for a new key
- `move_column_left` — move the column to the left
- `move_column_right` — move the column to the right
- `freeze_columns` — freeze the columns up to this one
- `search` — find text
- `search_next` — go to the next match
- `search_prev` — go to the previous match
- `quick_filter` — search: show only the rows containing some text
- `highlight_same` — highlight the rows with the same value
- `filter_same` — show only the rows with the same value
- `next_problem` — jump to the next problem
- `prev_problem` — jump to the previous problem
- `frequency` — show the value counts of the column
- `condition_1` — toggle the named condition with key 1
- `condition_2` — toggle the named condition with key 2
- `condition_3` — toggle the named condition with key 3
- `condition_4` — toggle the named condition with key 4
- `condition_5` — toggle the named condition with key 5
- `condition_6` — toggle the named condition with key 6
- `condition_7` — toggle the named condition with key 7
- `condition_8` — toggle the named condition with key 8
- `condition_9` — toggle the named condition with key 9
- `notice` — go to the notice box
- `sort_menu` — open the sort window
- `filter_menu` — open the filter window
- `workspace_drawer` — open or close the table list
- `workspace_switch` — open or close the list of favorite workspaces to switch to
- `workspace_favorite` — add the current workspace to favorites or remove it (in the switch list it only adds)
- `choose_0` — in the switch list, go back to where you started
- `choose_1` — in the switch list, go to favorite 1
- `choose_2` — in the switch list, go to favorite 2
- `choose_3` — in the switch list, go to favorite 3
- `choose_4` — in the switch list, go to favorite 4
- `choose_5` — in the switch list, go to favorite 5
- `choose_6` — in the switch list, go to favorite 6
- `choose_7` — in the switch list, go to favorite 7
- `choose_8` — in the switch list, go to favorite 8
- `choose_9` — in the switch list, go to favorite 9
- `mark_row` — mark or unmark the row
- `select_range` — select a range of rows
- `extend_up` — extend the selection up
- `extend_down` — extend the selection down
- `select_all` — select all rows
- `clear_selection` — clear the selection, the filter, the conditions, then the search highlight, one at a time
- `detail` — show every property of the note
- `view_settings` — open the view settings
- `focus_chips` — move to the settings bar
- `stack_down` — with stacked tables, move to the table below
- `stack_up` — with stacked tables, move to the table above
- `stack_close` — take the selected table out of the stack (also "Close this table" in the palette)
- `stack_grow` — with stacked tables, make the selected table one row taller
- `stack_shrink` — with stacked tables, make the selected table one row shorter
- `commit` — confirm the input
- `cancel` — cancel the input, or go back
- `commit_next` — confirm and move to the cell on the right
- `commit_prev` — confirm and move to the cell on the left
- `revert` — put back the value from before the edit
- `clear_input` — clear the whole input
- `list_up` — previous candidate
- `list_down` — next candidate
- `cursor_left` — move the cursor left
- `cursor_right` — move the cursor right
- `prev_month` — previous month in the date calendar
- `next_month` — next month in the date calendar
- `prev_year` — previous year in the date calendar
- `next_year` — next year in the date calendar
- `today` — set today's date (in the Gantt view, scroll the time axis to today)
- `create_note_edit` — create the note from the new note form and open it in the editor
- `create_note` — create the note now from any field of the new note form
- `time_focus` — switch between the day and the time field of the datetime calendar
- `cursor_home` — move the cursor to the start
- `cursor_end` — move the cursor to the end
- `delete_back` — delete the character before the cursor
- `delete_forward` — delete the character after the cursor
- `close` — close the help, palette, detail view, or settings bar
- `run` — run or choose the selected item
- `toggle` — turn the selected item on or off
- `next_section` — move to the next section
- `prev_section` — move to the previous section
- `go_to_section_1` — go to section 1 of the list on the left in the view settings
- `go_to_section_2` — go to section 2 of the list on the left in the view settings
- `go_to_section_3` — go to section 3 of the list on the left in the view settings
- `go_to_section_4` — go to section 4 of the list on the left in the view settings
- `go_to_section_5` — go to section 5 of the list on the left in the view settings
- `go_to_section_6` — go to section 6 of the list on the left in the view settings
- `go_to_section_7` — go to section 7 of the list on the left in the view settings
- `go_to_section_8` — go to section 8 of the list on the left in the view settings
- `go_to_section_9` — go to section 9 of the list on the left in the view settings
- `move_item_up` — move the item up
- `move_item_down` — move the item down
- `remove_item` — remove the item
- `save_all` — save all the changes
- `back` — go back without doing anything
- `next_file` — next file in the save review
- `prev_file` — previous file in the save review
- `overwrite` — write the file over changes made outside mdgrid
- `discard_row` — discard the pending changes to the file
- `quit_save` — save the changes, then quit
- `quit_discard` — discard the changes and quit
- `delete_note` — move the note of the selected row to the trash after confirming its name (also from the palette and the action menu `x`); in vim mode, `dd` and the palette move the notes of all marked rows; `u` brings them back
- `paste` — put the value copied with `y` or `Y` (vim mode)
- `repeat` — put the last value set in a single cell into the selected row (vim mode)
- `word_next` — move to the start of the next word (vim mode)
- `word_prev` — move to the start of the word or of the previous word (vim mode)
- `word_end` — move to the end of the word (vim mode)
- `insert` — insert before the cursor (vim mode)
- `append` — insert after the cursor (vim mode)
- `insert_start` — insert at the start (vim mode)
- `append_end` — insert at the end (vim mode)
- `delete_word` — delete to the start of the next word (vim mode)
- `delete_word_back` — delete back to the start of the word (vim mode)
- `delete_to_end` — delete to the end (vim mode)
- `change_word` — change to the end of the word (vim mode)
- `change_to_end` — change to the end (vim mode)
- `change_line` — change the whole text (vim mode)

### Palette-only commands

These commands have no key and can't be bound in the config file. Run them from the command palette.

- `export_base` — write mdgrid's views to a new `.base` file under a name you choose (existing `.base` files are never overwritten)
- `import_base` — pick a `.base` file and import one of its views as an mdgrid view
- `problems_only` — show only the rows with problems, or all rows again (also by clicking the problem count in the bottom band)
- `open_place` — list the registered tables (`places.toml`) and open the one you pick
- `register_place` — register the current table under a name and a group
- `save_session` — save what you see now (table, view, filters, sort, columns) as a named session in `sessions.toml`; open it later with `mdgrid -s <name>`. If the name exists, it asks before replacing
- `open_link` — open the note a link cell points to (its table, with its row selected)
- `linked_rows` — list the notes that link to the current note, with the table and column they link from
- `workspace_new` — create a workspace (`workspaces.toml`) with the current table
- `workspace_add` — add the current table to a workspace (pick one or type a new name)
- `workspace_remove` — remove the current table from a workspace
- `workspace_open` — pick a workspace, then one of its tables, and open it within that workspace
- `starter_new` — pick a starter (a built-in one or `starters/<name>` in the config folder), type the folder to create and whether to add the sample notes, and open the new ledger
- `rename_note` — rename the note of the selected row (also in the action menu `x`); links to the old name are rewritten as pending changes
- `restore_note` — pick a note in the trash and put it back where it was
- `across_search` — search every table in the workspace for a word and open the table of the row you pick (only with `across = true` under `[search]`; see [config.md](config.md#searchacross))
- `stack_add` — pick a table of the current scope and stack it below the current table (up to 4)
- `page_save` — name the stacked arrangement (tables, views, height ratios) and save it as a page of the current workspace
- `ai_proposals` — list the AI proposals for rows of this table (left in the ledger's `.mdgrid/proposals/` by `--apply … --propose`) and take one (as pending changes) or discard it (delete the proposal file). Shown only when there are proposals and mdgrid is not read-only; clicking the count in the bottom band does the same
- `routine_runs` — read the routine table again, list the routine runs that are due (including ones you left earlier that day) and create their notes after you confirm (only when a table of the scope has `runs` in its marker, and not when read-only; also by clicking the count in the bottom band; see [config.md](config.md#overrides-per-workspace-table-and-view))
