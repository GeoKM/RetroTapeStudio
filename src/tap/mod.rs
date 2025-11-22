//! TAP module: parses DEC-style `.TAP` records, detects formats, and surfaces tape data for downstream processing.
pub mod reader;
pub mod legacy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFormat {
    Raw,
    VmsBackup,
    Rsx11m,
    Rt11,
    RstsE,
}

#[cfg(test)]
mod tests;
