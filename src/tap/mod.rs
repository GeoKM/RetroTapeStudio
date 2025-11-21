pub mod reader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFormat {
    VmsBackup,
    Raw,
    Unknown,
}

#[cfg(test)]
mod tests;
