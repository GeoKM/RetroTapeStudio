//! TAP module: parses DEC-style `.TAP` records, detects formats, and surfaces `TapEntry` values for downstream processing.
pub mod reader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFormat {
    VmsBackup,
    Raw,
    Unknown,
}

#[cfg(test)]
mod tests;
