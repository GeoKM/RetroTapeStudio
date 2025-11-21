# RetroTapeStudio

RetroTapeStudio – GUI tool for inspecting and extracting data from DEC PDP-11 / VAX tape images (.TAP + .LOG).

## Overview
- Supports VMS BACKUP Phase-1 savesets and raw TAP data; RSX/RSTS detection is planned.
- Features: TAP record viewer, VMS BACKUP file browser, extraction to disk, log viewer, hex viewer, and save-set summary tab.
- Built in Rust with egui/eframe for a desktop UI.

## Getting Started
1. Build and run:
   - `cargo run`
2. Open media:
   - Use the Input/Contents tab to load a `.TAP` file and its companion `.LOG` (optional).
3. Browse and extract:
   - The Files tab shows parsed VMS files; click “Extract Files” (Extraction tab) to write payloads into a chosen directory.
4. Inspect records:
   - In Contents, click “View” to open the hex viewer for any TAP record.
5. Review status:
   - The Summary tab shows counts, formats, protections, and log warning/error totals.

## Architecture (high level)
- `tap` – TAP reader, detected formats, and record parsing.
- `backup::vms` – VMS BACKUP block and header parsers.
- `backup::extract` – Assembles VMS files, directory tree helpers.
- `log` – Parses `.LOG` files and correlates entries with TAP records.
- `summary` – Computes save-set level statistics.
- `gui` – Tabs for contents, files, extraction, log view, summary, shared state.
- `utils` – Helpers (e.g., hex formatting).

## Limitations / Roadmap
- Phase 12+: add RSX/RT-11/RSTS detection and handlers.
- Phase 13: broaden fixtures and verification coverage.
- Current focus is VMS BACKUP Phase-1; other formats are not yet parsed.

## License
MIT (see LICENSE when added).
