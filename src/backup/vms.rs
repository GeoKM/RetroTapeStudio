use std::convert::TryInto;

/// Parsed view of a VMS BACKUP Phase-1 block header and payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBlock {
    pub block_size: u16,
    pub format_version: u8,
    pub phase: u8,
    pub sequence_number: u32,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

/// Read a Phase-1 VMS BACKUP block from raw bytes.
///
/// The expected layout is:
/// - 0..2: little-endian block size (bytes within this block).
/// - 2: format version byte.
/// - 3: phase identifier (must be 1 for Phase-1 blocks).
/// - 4..8: little-endian sequence number.
/// - 8..10: little-endian checksum field (exposed but not validated here).
/// - 10..N: payload bytes recorded by BACKUP.
pub fn read_backup_block(data: &[u8]) -> Result<BackupBlock, String> {
    const HEADER_LEN: usize = 10;

    if data.len() < HEADER_LEN {
        return Err("buffer too small for VMS BACKUP header".into());
    }

    let block_size = u16::from_le_bytes([data[0], data[1]]);
    if block_size == 0 {
        return Err("block size must be greater than zero".into());
    }

    let block_size_usize = block_size as usize;
    if block_size_usize > data.len() {
        return Err(format!(
            "block size {} exceeds available data {}",
            block_size, data.len()
        ));
    }

    let format_version = data[2];
    let phase = data[3];
    if phase != 1 {
        return Err(format!("unsupported phase {}, expected Phase-1", phase));
    }

    let sequence_number = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let checksum = u16::from_le_bytes([data[8], data[9]]);

    let payload = data[HEADER_LEN..block_size_usize].to_vec();

    Ok(BackupBlock {
        block_size,
        format_version,
        phase,
        sequence_number,
        checksum,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::read_backup_block;

    #[test]
    fn parses_minimal_block() {
        let mut raw = vec![0u8; 12];
        raw[0..2].copy_from_slice(&12u16.to_le_bytes());
        raw[2] = 2; // format version
        raw[3] = 1; // phase
        raw[4..8].copy_from_slice(&5u32.to_le_bytes());
        raw[8..10].copy_from_slice(&0xABCDu16.to_le_bytes());
        raw[10] = 0x11;
        raw[11] = 0x22;

        let block = read_backup_block(&raw).expect("should parse");

        assert_eq!(block.block_size, 12);
        assert_eq!(block.format_version, 2);
        assert_eq!(block.phase, 1);
        assert_eq!(block.sequence_number, 5);
        assert_eq!(block.checksum, 0xABCD);
        assert_eq!(block.payload, vec![0x11, 0x22]);
    }

    #[test]
    fn rejects_non_phase_one() {
        let mut raw = vec![0u8; 10];
        raw[0..2].copy_from_slice(&10u16.to_le_bytes());
        raw[3] = 2;

        let err = read_backup_block(&raw).unwrap_err();
        assert!(err.contains("Phase-1"));
    }
}
