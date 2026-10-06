# Obsidian Bases support

[日本語](obsidian-bases.ja.md)

mdgrid opens an Obsidian Bases file (`.base`) and shows its table views. `.base` has no published grammar and its meaning can change between Obsidian versions, so mdgrid aims to get the commonly used parts of the table view right and to say plainly when it meets something it does not understand. It never guesses rows or values.

This page lists what mdgrid reads and what it does not. The lists of functions, methods and `file` properties below are checked by tests against the expression parser (`tests/test_bases_docs.rs`): adding a function to the parser without listing it here, or listing a function the parser rejects, fails the tests.

mdgrid never changes an existing `.base` file. Exporting a view writes a new `.base` file under a name you choose instead. Unknown keys are ignored rather than reported as errors.

## Which notes are searched

The rows come from the vault the `.base` belongs to. mdgrid finds the vault root by walking up from the `.base` file's folder to the nearest folder that contains `.obsidian/`, then reads every note under it. If no `.obsidian/` folder is found (for example, a vault copied without its settings), only the `.base` file's own folder is searched, so a filter such as `file.inFolder("Projects")` can match nothing. When that happens and the view is empty, mdgrid says which folder it searched (on the bottom line, and on stderr with `--print`). To fix it, create an empty `.obsidian/` folder at the vault root.

Like Obsidian, mdgrid does not look inside folders whose names start with `.` (such as `.obsidian`, `.git` and `.trash`). It also skips `node_modules`, so opening a code repository does not list the documentation of its dependencies. A folder you name on the command line is always searched, even if its name starts with `.`.

## Keys read

Top level:

| Key | What mdgrid does with it |
|---|---|
| `filters` | Applied to every view, combined with the view's own `filters` by AND. |
| `formulas` | Named expressions. Each one can be shown as a column `formula.<name>`, used for sorting and grouping, and referenced from other expressions. |
| `properties` | Only `displayName`, which becomes the column title. Keys may be written as `status` or `note.status`. Columns without a `displayName` get Obsidian's default titles (a formula column shows the formula's name, `file.name` shows `file name`, `file.ctime` shows `created time`, and so on). The keys of `--print --format json` use the same titles. |
| `views` | The list of views. Each view becomes a tab; the first view opens at startup. |
| `summaries` | Formula summaries (such as `values.mean()`) are not evaluated. A view that uses one still opens, and the bottom line names it as unsupported. See [Summaries](#summaries). |

Inside a view:

| Key | What mdgrid does with it |
|---|---|
| `type` | Must be `table`. See [View types](#view-types). |
| `name` | The tab name. |
| `filters` | The view's own filters. |
| `order` | The columns, in order: note properties (`status` or `note.status`), `file.*` and `formula.*`. Without `order`, the columns are `file.name` followed by every frontmatter key found in the notes. |
| `sort` | A list of `property` and `direction` (`ASC` or `DESC`, case-insensitive; anything else means `ASC`). Earlier entries win. |
| `groupBy` | `property` and `direction`. Rows are gathered under one heading per value; empty values go under `(empty)` (`(空)` in Japanese). Within a group, rows follow `sort`. |
| `limit` | A whole number. Applied last, to the total number of rows. |
| `summaries` | A map from a column (`estimate`, `note.estimate`, `file.*` or `formula.*`) to a built-in summary name. See [Summaries](#summaries). |

`filters` (both at the top level and in a view) is either an expression string or a map with `and`, `or` and `not`, each holding a list of expression strings or further maps. Several of these keys in one map are combined by AND. `not` keeps a row when none of its entries is true. Any other key in such a map, or a value of another shape, makes the view fail to open (see [Not supported](#not-supported)). A filter keeps a row only when its expression is true under the same rules as `if`: `null`, `false`, `0`, and an empty string are false.

Sorting by a note property puts values that fit the column's type first (in the given direction), then values that do not fit (by text, A to Z), then empty values, whatever the direction. Sorting by `formula.*` or `file.*` has no column type: values of different types are ordered by type (booleans, numbers, durations, dates and date-times, strings, lists), and `DESC` reverses that whole order; empty values (`null`, an empty string, an empty list) still come last. Rows with equal keys keep their original order.

Everything else in the file (for example `columnSize`, `rowHeight`, and keys mdgrid does not know) is ignored, but never removed, because mdgrid does not change the file.

## View types

Only `table` is supported. A view with another `type` (`cards`, `list`, `map`, a plugin's view, ...) or with no `type` does not open: its tab is still listed, the table area shows the reason, for example `cards view "Gallery" is not supported (mdgrid shows table views only)`, and the bottom line tells you to switch with `[` `]`. mdgrid does not try to draw these views as a table.

## Operators

| Operator | Meaning |
|---|---|
| `==` `!=` | Equal and not equal, for any type. Values of different types are not equal, except dates (see below). `===` and `!==` are read as `==` and `!=`. |
| `<` `<=` `>` `>=` | Order of numbers, strings, booleans, durations, and dates and date-times. A duration compares with a number as milliseconds. Other mixes of types give `null`. |
| `&&` `\|\|` `!` | And, or, not, using the truthiness rules of `if`. |
| `+` `-` `*` `/` `%` | Arithmetic on numbers. `+` also joins strings (and a string with a number). Division by zero and overflow give `null`. |
| `-x` `+x` | Unary minus and plus. |
| `( )` | Grouping. `*` `/` `%` bind tighter than `+` `-`, then comparisons, then `==` `!=`, then `&&`, then `\|\|`. Operators of the same level group from the left. |
| `x[i]` | Item `i` (counted from 0) of a list. |

Date arithmetic:

- date `+` or `-` a duration string, such as `due + "1d"`, `today() - "2 weeks"`, or `now() + "1d 3h"`. Units: `y`/`year`, `M`/`month`, `w`/`week`, `d`/`day`, `h`/`hour`, `m`/`min`/`minute`, `s`/`sec`/`second` (plurals and `yr`, `hr`, `hrs`, `mins`, `secs` also work). Months and years follow the calendar; a day past the end of the month becomes the last day of that month.
- date `+` or `-` a value from `duration()`, such as `due + duration("1d")`. `duration()` values can also be added to and subtracted from each other.
- date `-` date gives a duration in milliseconds.
- duration `*` or `/` a number gives a duration (a duration under one second shows as `ms`). To get a number of days, use `number()`: `number(due - today()) / 86400000`.

Values:

- Literals: strings in `"..."` or `'...'` (with `\n`, `\t`, `\r` escapes), numbers, `true`, `false`, `null`, and lists `[a, b]`.
- References: a bare name or `note.name` is a frontmatter key; `note["a b"]` reaches a key with spaces. `formula.name` (or `formula["name"]`) is a formula. `file.name` and the rest are listed under [File properties](#file-properties). A missing key gives `null`.
- A frontmatter string in the form `YYYY-MM-DD` is read as a date, and `YYYY-MM-DDTHH:MM(:SS)` as a date-time in local time, as Obsidian does. `today()` and `now()` also use the local time zone. When one side of a comparison or of `+`/`-` is a date and the other a string, the string is read as a date too.

An operation on types that do not fit gives `null` instead of stopping.

## Functions

- `if(condition, then, else)`: `then` when `condition` is true, otherwise `else`. `else` may be left out, giving `null`.
- `date(text)`: reads `YYYY-MM-DD` as a date and `YYYY-MM-DDTHH:MM(:SS)` as a date-time; a date passes through; anything else gives `null`.
- `now()`: the current date and time.
- `today()`: today's date.
- `duration(text)`: reads a duration string such as `"1d"` or `"2 weeks"` as milliseconds (a month counts as 30 days and a year as 365).
- `number(value)`: a duration as milliseconds, a date or date-time as milliseconds since 1970-01-01, `true`/`false` as 1/0, and text that reads as a number as that number; anything else is `null`.
- `max(a, b, ...)`: the largest of the numbers; `null` if any value is not a number.
- `min(a, b, ...)`: the smallest of the numbers; `null` if any value is not a number.
- `list(value)`: a list stays as it is; any other value becomes a list with that one item.

## Methods

- `contains(value)`: for a string, whether it contains the text; for a list, whether it has an equal item.
- `containsAll(a, b, ...)`: whether every value is contained.
- `containsAny(a, b, ...)`: whether at least one value is contained.
- `isEmpty()`: true for `null`, an empty string and an empty list.
- `toString()`: the value as text.
- `lower()`: a string in lower case; `null` for anything else.
- `upper()`: a string in upper case, so `status.upper() == "ACTIVE"` works as a filter.
- `title()`: a string with the first letter of each word in upper case and the rest in lower case.
- `trim()`: a string without white space at both ends.
- `startsWith(text)`: whether a string starts with the text.
- `endsWith(text)`: whether a string ends with the text.
- `slice(start, end)`: part of a string or a list, from `start` up to (not including) `end`. `end` may be left out; negative numbers count from the end.
- `replace(pattern, replacement)`: a string with every occurrence of the text `pattern` replaced. Regular expressions are not supported.
- `split(separator, n)`: a string cut at the text `separator` into a list (at most `n` items when given). Regular expressions are not supported.
- `reverse()`: a list in reverse order, or a string with its characters reversed.
- `round(digits)`: a number rounded to the nearest whole number, or to `digits` decimal places.
- `floor()`: a number rounded down.
- `ceil()`: a number rounded up.
- `abs()`: the absolute value of a number.
- `date()`: a date or date-time as a date (the time dropped).
- `time()`: the time of a date or date-time as `HH:mm:ss` (`00:00:00` for a date).
- `format(pattern)`: a date or date-time as text. The pattern uses Moment's tokens (English names): `YYYY` `YY` `Q` `MMMM` `MMM` `MM` `M` `DDDD` `DDD` `DD` `Do` `D` `dddd` `ddd` `dd` `d` `e` `E` `HH` `H` `hh` `h` `A` `a` `mm` `ss` `SSS` `X` `x`, ISO weeks `GGGG` `WW` `W`, and Sunday-start weeks `gggg` `ww` `w` (the week with January 1 is week 1). Other characters, and anything inside `[...]`, are copied as they are.
- `join(separator)`: the items of a list as text joined by the separator.
- `unique()`: a list without repeated items (the first of each is kept).
- `sort()`: a list from smallest to largest.
- `flat()`: a list with the lists inside it opened up by one level.
- `length`: the number of characters of a string or items of a list, written without parentheses (`tags.length`).
- `year`: the year of a date or date-time, written without parentheses (`due.year`). The same goes for the five below.
- `month`: the month (1 to 12).
- `day`: the day of the month.
- `hour`: the hour (0 for a date).
- `minute`: the minute (0 for a date).
- `second`: the second (0 for a date).

## File properties

Values:

- `file.name`: the file name with its extension.
- `file.basename`: the file name without the extension.
- `file.ext`: the extension.
- `file.path`: the path from the vault root.
- `file.folder`: the folder from the vault root.
- `file.size`: the size in bytes.
- `file.mtime`: the time the file was last modified.
- `file.ctime`: the time the file was created.
- `file.tags`: the tags of the note: the frontmatter `tags` (also `tag`, `Tags`, in any case; a list, or text split at commas and spaces) and the `#tags` in the body.
- `file.links`: the notes this note links to, as a list. Links are taken from the body and from frontmatter values: `[[name]]`, `[[name|text]]`, `[[name#heading]]`, `![[name]]` and `[text](path.md)`. Links inside code blocks and inline code are skipped, as are web addresses. Each link becomes the path of the target note from the vault root without `.md` (for example `sub/c`). The target is the note at that path from the vault root (a `[text](path)` link first tries the path relative to the note's folder); otherwise the note with that name, and when several notes share the name, the one nearest the vault root (then the shortest path). Case does not matter. A link that matches no note stays as written (`[[missing]]` gives `missing`). Each target is listed once.
- `file.backlinks`: the notes that link to this note, as a list of their paths without `.md` (the same link rules as `file.links`).

Functions on `file`:

- `file.hasTag(tag, ...)`: true when the note has any of the tags. A leading `#` is ignored, case does not matter, and a nested tag `a/b` also matches `a`.
- `file.inFolder(folder)`: true when the note is in that folder or one of its subfolders.
- `file.hasProperty(name)`: true when the frontmatter has that key.
- `file.hasLink(note)`: true when the note links to `note` (a name, a path, or `this.file`).

## this

When a `.base` file is opened directly, `this` is that `.base` file. `this.file` gives its path, and `this.file.name`, `this.file.basename`, `this.file.ext`, `this.file.path`, `this.file.folder`, `this.file.size`, `this.file.mtime` and `this.file.ctime` give the same values as for a note (`this.file.tags` is an empty list). For example, `file.inFolder(this.file.folder)` keeps the notes next to the `.base` file, and `file.links.contains(this.file)` keeps the notes that link to it (`[[Projects.base]]` or `[[P/Projects.base]]`: a link to the `.base` file by its name or path gives the same value as `this.file`). Where there is no `.base` file to point at (a mdgrid view of a folder), an expression with `this` is not supported.

## Summaries

A view's `summaries` adds one row under the table, just above the bottom line. The left column says `Summary` (`集計` in Japanese), and each summarized column shows the summary name and its value, for example `Sum 18` or `Earliest 2026-10-05`, lined up with the column and scrolled with it. Summaries are computed from the rows the view shows now, after its filters, the view settings and the quick filter, so narrowing the table changes the values. Pending (unsaved) values count. A view without `summaries` has no summary row. In Japanese the names are shown in Japanese (`合計 18`).

```yaml
views:
  - type: table
    name: Open
    summaries:
      estimate: Sum
      due: Earliest
      done: Checked
```

The built-in summaries, as in Obsidian. Values of another type are not counted. When there is nothing to count, a number or date summary is blank; the counting summaries show `0`.

| Name | Counts | Value |
|---|---|---|
| `Average` | numbers | the mean |
| `Min` | numbers | the smallest number |
| `Max` | numbers | the largest number |
| `Sum` | numbers | the sum |
| `Range` | numbers, or dates | `Max` − `Min`; for a column of dates, `Latest` − `Earliest` as a duration |
| `Median` | numbers | the middle number (the mean of the two middle numbers when the count is even) |
| `Stddev` | numbers | the population standard deviation (divided by the count) |
| `Earliest` | dates and date-times | the earliest |
| `Latest` | dates and date-times | the latest |
| `Checked` | booleans | the number of `true` |
| `Unchecked` | booleans | the number of `false` |
| `Empty` | any | the number of empty values (`null`, a missing key, an empty string, an empty list) |
| `Filled` | any | the number of values that are not empty |
| `Unique` | any | the number of different values that are not empty (the number `1` and the string `"1"` differ) |

## Not supported

mdgrid does not evaluate the following. It never guesses a result for them.

- Every other function and method from Obsidian, for example `link()`, `icon()`, `image()`, and methods such as `map()`, `filter()`, `reduce()`, `relative()`, `toFixed()`, `repeat()`, `asFile()`, and `linksTo()`, and a regular expression given to `replace()` or `split()`. Obsidian's list may grow; anything not listed on this page is treated as unsupported.
- `file` properties `file.embeds`, `file.properties` and `file.file`, and the function `link()`.
- `this` other than `this.file` and its values listed under [this](#this), and `this` for a base embedded in a note (mdgrid only opens `.base` files).
- Formula summaries: the top-level `summaries` (for example `customAverage: 'values.mean()'`) and a view summary that names one, or any name that is not a built-in summary. The view still opens; the bottom line lists it after `unsupported:` (`未対応:` in Japanese) and the other summaries are shown.
- Summaries per group. With `groupBy`, the summary row is for the whole table, and the bottom line says so.
- Views other than `table`: `cards`, `list`, `map`, and plugin views.
- `base` code blocks inside notes. mdgrid only opens `.base` files.

What you see when mdgrid meets one of these (the examples are the English messages; the Japanese screen says the same in Japanese):

- In `filters` (top level or view): the view does not open. The table area shows the reason with the name, for example `(this view cannot be opened: cannot evaluate the filters of view "Open": unsupported function or field .relative() (expression: due.relative() == "yesterday"))`, and the bottom line says the view cannot be opened. Other views still open; switch with `[` `]`.
- In a column (`formula.*` or `file.*` in `order`): every cell of that column shows a dim `?` instead of a value and cannot be edited. The dim `?` is distinct from an empty cell (missing key) and from a real `?` value. Selecting such a cell shows the unsupported expression on the bottom line (so you can tell even when the terminal cannot draw dim text), and the column is listed after `unsupported:` on the bottom line.
- In `sort` or `groupBy`: the key is treated as empty for every row, and the bottom line notes that the view cannot be sorted by it.
- A formula that refers to a missing formula or to itself in a loop counts as unsupported in the same way.
- A syntax error in an expression is reported the same way, with the reason instead of a name.
