use crate::tap::reader::read_tap_entry;
use crate::TapeError;

#[test]
fn short_read_errors() {
    let err = read_tap_entry(&[]).unwrap_err();
    assert!(matches!(err, TapeError::Parse(_)));
}

#[test]
fn overly_large_record_rejected() {
    let data = vec![0u8; 1_048_577];
    let err = read_tap_entry(&data).unwrap_err();
    assert!(matches!(err, TapeError::UnsupportedFormat(_)));
}

#[test]
fn silent_eof_reports_error() {
    // Declares a block size larger than the buffer, triggering a parse error.
    let mut data = vec![0u8; 32];
    data[0..2].copy_from_slice(&64u16.to_le_bytes());
    data[2] = 1;
    data[3] = 1;
    let entry = read_tap_entry(&data).expect("should fall back to raw");
    assert!(matches!(
        entry.kind,
        crate::tap::reader::TapDataKind::Raw(_)
    ));
}
