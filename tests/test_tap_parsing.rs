use retro_tape_studio_v6_safe::backup::vms::BackupBlock;
use retro_tape_studio_v6_safe::tap::reader::{read_tap_entry, TapDataKind};
use retro_tape_studio_v6_safe::tap::DetectedFormat;

mod common;
use common::{load_tap_fixture, read_tap_file_with_chunks};

fn make_vms_block() -> Vec<u8> {
    let mut raw = vec![0u8; 32];
    raw[0..2].copy_from_slice(&32u16.to_le_bytes());
    raw[2] = 1;
    raw[3] = 1;
    raw[4..8].copy_from_slice(&1u32.to_le_bytes());
    raw[8..10].copy_from_slice(&0xAAAAu16.to_le_bytes());
    raw[10] = 0x11;
    raw
}

#[test]
fn detects_vms_backup() {
    let entry = read_tap_entry(&make_vms_block()).expect("parse");
    assert!(matches!(entry.kind, TapDataKind::VmsBlock(BackupBlock { .. })));
    assert_eq!(entry.detected_format, DetectedFormat::VmsBackup);
}

#[test]
fn detects_rsx_rt11_rsts() {
    let rsx = load_tap_fixture("rsx.tap");
    let rt11 = load_tap_fixture("rt11.tap");
    let rsts = load_tap_fixture("rsts.tap");

    let rsx_entry = read_tap_file_with_chunks(&rsx).unwrap()[0].detected_format;
    let rt11_entry = read_tap_file_with_chunks(&rt11).unwrap()[0].detected_format;
    let rsts_entry = read_tap_file_with_chunks(&rsts).unwrap()[0].detected_format;

    assert_eq!(rsx_entry, DetectedFormat::Rsx11m);
    assert_eq!(rt11_entry, DetectedFormat::Rt11);
    assert_eq!(rsts_entry, DetectedFormat::RstsE);
}

#[test]
fn falls_back_to_raw() {
    let data = vec![1u8, 2, 3, 4];
    let entry = read_tap_entry(&data).unwrap();
    assert_eq!(entry.detected_format, DetectedFormat::Raw);
}
