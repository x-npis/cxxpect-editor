# Architecture

The MVP deliberately owns one `Document`. It records the path, UTF-8 text, saved snapshot, detected line ending, dirty state, and monotonically increasing revision.

## Verification

Every edit schedules verification after a 300 ms debounce. The UI sends `{ revision, path, text }` through a channel to a dedicated worker thread. The worker constructs a `cxxpect::SourceFile`, calls `cxxpect::verify`, and returns the source, diagnostics, and revision. The UI applies a result only when its revision equals the document revision.

Diagnostics retain the matching `SourceFile`. Navigation always calls `SourceFile::locate(Diagnostic::span)` and uses its Unicode `char_offset`; byte offsets are never treated as editor cursor positions.

## Modules

- `document`: file and dirty/revision state.
- `verification`: debounce, worker thread, and stale-result rejection.
- `editor`: `TextEdit`, shared gutter scrolling, highlighting, and cursor status.
- `diagnostics_panel`: stable resizable diagnostics list and navigation.
- `commands`: command vocabulary shared by menus, toolbar, and shortcuts.
- `settings`: persisted theme and panel preference.
- `app`: orchestration, dialogs, drag-and-drop, and close confirmation.

The editor imports only symbols exported from the root of the `cxxpect` crate; no core changes are required.
