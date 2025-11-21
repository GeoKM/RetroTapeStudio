# Architecture

## Data Flow
1. TAP reader (`tap::reader`) parses records into `TapEntry` values, tagging a `DetectedFormat`.
2. VMS BACKUP parsing (`backup::vms`) decodes FH2/XH2/directory records; `backup::extract` assembles `VmsFile` lists and directory trees.
3. Log parsing (`log::parse`) reads companion `.LOG` files and correlates warnings/errors back to `TapEntry` items.
4. Summary (`summary::compute_saveset_summary`) aggregates counts, histograms, efficiency, and log metadata.
5. GUI (`gui::*`) renders Contents, Files, Extraction, Log, and Summary tabs from shared `AppState`.

## AppState
- `TapState` (entries + selected_entry) stores parsed TAP records.
- `LogState` holds parsed `.LOG` data and whether correlation has occurred.
- Extraction and summary fields remain as captured state; files and directory trees live alongside.

## TAP / LOG / Summary Relationship
- Loading a `.TAP` populates `TapState.entries`.
- Loading a `.LOG` triggers correlation, marking `TapEntry.log_level` where matching record numbers are found.
- Summary consumes both parsed files and log metadata to report totals, protections, formats, and warning/error counts.

## Error Handling
- Shared `TapeError`/`TapeResult` encode IO, parse, and unsupported-format errors; parser functions return these directly.

## Extending Formats
- New format recognizers (e.g., RSX/RT-11/RSTS) should plug in at `tap` (format detection) and feed new extractors modeled after `backup::extract`. GUI tabs can display new structures alongside existing views without altering the TAP reader contract.
