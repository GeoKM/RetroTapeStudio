# Testing

## Running Tests
- `cargo test` runs the full unit test suite (TAP parsing, VMS parsing, extraction, log parsing, summary helpers).

## Coverage
- TAP: record parsing, error paths, detected format flags.
- VMS: block/header parsers and assembly helpers.
- Extraction: grouping into files and directory trees.
- Log: parsing, correlation with TAP entries.
- Summary: exercised indirectly through component tests.

## Adding New Fixtures
- Future sample media will live under `tests/data/`:
  - `tests/data/vms` – VMS BACKUP savesets
  - `tests/data/rsx` – RSX/RT-11 images
  - `tests/data/rsts` – RSTS images
  - `tests/data/logs` – log files
- Phase 13 will add comprehensive fixtures for VMS/RSX/RSTS once the new format handlers land.
