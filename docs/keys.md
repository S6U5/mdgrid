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
- Modes where you type text (`edit`, `palette`, `search`, `filter`, `settings_input`, `list_select`) don't bind single letters by default and don't accept key sequences.

In the `table` mode, full-width letters and symbols typed while a Japanese input method is on act like their half-width keys, and `、` and `・` act like `,` and `/`.

## `table`

The main grid of notes.

When you start mdgrid with `--pick`, `Enter` on a note row prints the marked rows (or the selected row) and quits, `q` cancels, and `Esc` clears the selection, filters, and search as usual, cancelling only when there is nothing left to clear. `Enter` on a group heading row still expands or collapses it, and `e` does not open the editor.

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
| `a` | `new_note` | Create a new note |
| `Ctrl+s` | `save` | Review the pending changes as a diff, then save them |
| `q` | `quit` | Quit (asks first if there are unsaved changes) |
| `Ctrl+g` | `cancel_load` | Stop loading notes |
| `?` | `help` | Show the help with every key |
| `:` | `palette` | Open the command palette (also takes a row number like `:120`, `w` to save, `q` to quit) |
| `Ctrl+p` | `palette` | Open the command palette |
| `x` | `action_menu` | Open the menu of actions you can use on the selected cell, each with its current key (right-clicking a cell also opens it) |
| `e` | `open_editor` | Open the selected note in your editor, and reload it when you return |
| `y` | `copy` | Copy the selected cell (or the selected rows) to the clipboard |
| `Ctrl+c` | `copy` | Copy the selected cell (or the selected rows) to the clipboard |
| `Y` | `copy_row` | Copy the whole row (or the selected rows) to the clipboard |
| `[` | `prev_view` | Switch to the previous view |
| `]` | `next_view` | Switch to the next view |
| `<` | `narrow_column` | Make the column narrower |
| `>` | `widen_column` | Make the column wider |
| `s` | `sort_column` | Sort by this column temporarily: ascending, descending, then off (the `.base` file is not changed) |
| `-` | `hide_column` | Hide the column |
| `+` | `show_column` | Bring back the last hidden column |
| `R` | `relation_map` | Open the relation map of the registered tables |
| `A` | `add_column` | Add a column for a key no note has yet (type its name); saving writes the key only to the notes you fill in |
| `H` | `move_column_left` | Move the column to the left |
| `L` | `move_column_right` | Move the column to the right |
| `F` | `freeze_columns` | Freeze the columns up to this one so they stay visible when scrolling (press again to unfreeze) |
| `/` | `search` | Search for text |
| `n` | `search_next` | Go to the next match |
| `N` | `search_prev` | Go to the previous match |
| `\` | `quick_filter` | Show only the rows containing some text (quick filter) |
| `*` | `highlight_same` | Highlight the rows with the same value as the selected cell |
| `,` | `filter_same` | Show only the rows with the same value as the selected cell |
| `%` | `frequency` | Show how many rows have each value of the selected column, and pick one to show only those rows |
| `Space` | `mark_row` | Mark or unmark the row, then move down |
| `v` | `select_range` | Start selecting a range of rows; press again to mark the range |
| `Shift+Up` | `extend_up` | Extend the selection up |
| `Shift+Down` | `extend_down` | Extend the selection down |
| `Ctrl+a` | `select_all` | Select all rows |
| `Esc` | `clear_selection` | Clear the selection; if nothing is selected, clear the filter, then the search highlight |
| `K` | `detail` | Show every property of the selected note, and the first lines of its body |
| `o` | `view_settings` | Open the view settings (filters, sort, grouping, columns) |
| `f` | `focus_chips` | Move to the settings bar above the table |

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

The detail view, which lists every property of one note and, under a `Body` heading, the first 20 lines of its body (read-only; no heading when the note has no body).

| Key | Action | Description |
|---|---|---|
| `Enter` | `edit` | Edit the selected property |
| `Esc` | `close` | Close the detail view |
| `K` | `close` | Close the detail view |
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

The help screen (`?`).

| Key | Action | Description |
|---|---|---|
| `Esc` | `close` | Close the help |
| `q` | `close` | Close the help |
| `?` | `close` | Close the help |
| `j` | `down` | Scroll down |
| `Down` | `down` | Scroll down |
| `k` | `up` | Scroll up |
| `Up` | `up` | Scroll up |
| `PageDown` | `page_down` | Scroll down one screen |
| `PageUp` | `page_up` | Scroll up one screen |
| `g g` | `top` | Go to the top |
| `G` | `bottom` | Go to the bottom |

## `search`

Typing a search term (`/`).

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Search for the term |
| `Esc` | `cancel` | Cancel the search |
| `Backspace` | `delete_back` | Delete the character before the cursor |

## `filter`

Typing a quick filter term (`\`).

| Key | Action | Description |
|---|---|---|
| `Enter` | `commit` | Apply the filter |
| `Esc` | `cancel` | Clear the filter |
| `Backspace` | `delete_back` | Delete the character before the cursor |

## `settings`

The view settings screen (`o`), where you change filters, sorting, grouping, and columns.

| Key | Action | Description |
|---|---|---|
| `Enter` | `run` | Choose the selected item |
| `Space` | `toggle` | Turn the selected item on or off |
| `Esc` | `cancel` | Go back, or cancel |
| `Tab` | `next_section` | Move to the next section |
| `Shift+Tab` | `prev_section` | Move to the previous section |
| `j` | `down` | Move down |
| `Down` | `down` | Move down |
| `k` | `up` | Move up |
| `Up` | `up` | Move up |
| `h` | `left` | Move left |
| `Left` | `left` | Move left |
| `l` | `right` | Move right |
| `Right` | `right` | Move right |
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

## `list_select`

Picking values for a list property. Letter keys type into the search box.

| Key | Action | Description |
|---|---|---|
| `Tab` | `commit` | Confirm the selection |
| `Enter` | `run` | Confirm the selection; while searching, add the highlighted candidate and clear the search |
| `Space` | `toggle` | Add or remove the highlighted candidate; while searching, type a space |
| `Esc` | `cancel` | Cancel |
| `Ctrl+r` | `revert` | Put back the selection from when the list was opened |
| `Ctrl+s` | `create_note` | In the new note form, create the note now from any field |
| `Ctrl+e` | `create_note_edit` | In the new note form, create the note and open it in the editor |
| `Up` | `list_up` | Previous candidate |
| `Down` | `list_down` | Next candidate |
| `Ctrl+p` | `list_up` | Previous candidate |
| `Ctrl+n` | `list_down` | Next candidate |
| `PageUp` | `page_up` | Scroll up one screen |
| `PageDown` | `page_down` | Scroll down one screen |
| `Backspace` | `delete_back` | Delete the character before the cursor |

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

## `relations`

The relation map (`R`, `:relation_map`, or the **Relations** tab at the top right, which appears once you have registered tables). It shows the registered tables (and the current one) as boxes with their row counts and columns, and draws an arrow from each column that links to another table: `N` on the linking side, `1` (many-to-one) or `N` (many-to-many, a list of links) on the target side. The panes follow the terminal size: 120 columns or more show the tables, the map and the details side by side; 80 or more show the map and the details; narrower terminals show the tables and links as a list. With 30 rows or more, the linked records of the selected link appear below.

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
| `?` | `help` | Show help |

Clicking the **Table** tab at the top right goes back to the table.

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
- `save` — review and save the pending changes
- `quit` — quit mdgrid
- `cancel_load` — stop loading notes
- `help` — show the help
- `palette` — open the command palette
- `action_menu` — open the menu of actions for the selected cell
- `open_editor` — open the note in your editor
- `copy` — copy the cell or the selected rows
- `copy_row` — copy the row or the selected rows
- `prev_view` — switch to the previous view
- `next_view` — switch to the next view
- `narrow_column` — make the column narrower
- `widen_column` — make the column wider
- `sort_column` — sort by the column temporarily
- `hide_column` — hide the column
- `show_column` — bring back the last hidden column
- `relation_map` — switch between the table and the relation map
- `add_column` — add a column for a new key
- `move_column_left` — move the column to the left
- `move_column_right` — move the column to the right
- `freeze_columns` — freeze the columns up to this one
- `search` — search for text
- `search_next` — go to the next match
- `search_prev` — go to the previous match
- `quick_filter` — show only the rows containing some text
- `highlight_same` — highlight the rows with the same value
- `filter_same` — show only the rows with the same value
- `frequency` — show the value counts of the column
- `mark_row` — mark or unmark the row
- `select_range` — select a range of rows
- `extend_up` — extend the selection up
- `extend_down` — extend the selection down
- `select_all` — select all rows
- `clear_selection` — clear the selection or the filter
- `detail` — show every property of the note
- `view_settings` — open the view settings
- `focus_chips` — move to the settings bar
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
- `today` — set today's date
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

### Palette-only commands

These commands have no key and can't be bound in the config file. Run them from the command palette.

- `export_base` — write mdgrid's views to a new `.base` file under a name you choose (existing `.base` files are never overwritten)
- `import_base` — pick a `.base` file and import one of its views as an mdgrid view
- `open_place` — list the registered tables (`places.toml`) and open the one you pick
- `register_place` — register the current table under a name and a group
- `open_link` — open the note a link cell points to (its table, with its row selected)
- `linked_rows` — list the notes that link to the current note, with the table and column they link from
- `workspace_new` — create a workspace (`workspaces.toml`) with the current table
- `workspace_add` — add the current table to a workspace (pick one or type a new name)
- `workspace_remove` — remove the current table from a workspace
- `workspace_open` — pick a workspace, then one of its tables, and open it within that workspace
