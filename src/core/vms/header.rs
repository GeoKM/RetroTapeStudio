use super::*;

#[derive(Debug, Clone)]
pub struct VmsFileHeader {
    pub filename: String,
    pub version: u16,
    pub uic: (u16, u16),
    pub record_format: u8,
    pub record_attributes: u16,
    pub block_count: u32,
}

impl VmsFileHeader {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 128 {
            return None;
        }

        let name_len = raw[32] as usize;
        let ext_len = raw[33] as usize;

        let mut name = String::new();
        name.push_str(std::str::from_utf8(&raw[34..34 + name_len]).ok()?);
        if ext_len > 0 {
            name.push('.');
            name.push_str(std::str::from_utf8(&raw[34 + name_len..34 + name_len + ext_len]).ok()?);
        }

        let version = u16::from_le_bytes([raw[64], raw[65]]);
        let uic = (
            u16::from_le_bytes([raw[90], raw[91]]),
            u16::from_le_bytes([raw[92], raw[93]]),
        );

        let rfm = raw[80];
        let rat = u16::from_le_bytes([raw[81], raw[82]]);
        let blocks = u16::from_le_bytes([raw[108], raw[109]]) as u32;

        Some(Self {
            filename: name,
            version,
            uic,
            record_format: rfm,
            record_attributes: rat,
            block_count: blocks,
        })
    }
}
