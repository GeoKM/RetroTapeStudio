use retro_tape_studio_v6_safe::backup::vms::BackupBlock;
use retro_tape_studio_v6_safe::tap::reader::{read_tap_entry, TapDataKind};
use retro_tape_studio_v6_safe::tap::DetectedFormat;

mod common;
use common::{load_tap_fixture, read_tap_file_with_chunks, read_tap_with_chunks, write_output};

fn make_vms_block() -> Vec<u8> {
    let mut raw = vec![0u8; 80];
    raw[0..2].copy_from_slice(&80u16.to_le_bytes());
    raw[2] = 2;
    raw[3] = 1;
    raw[4..8].copy_from_slice(&1u32.to_le_bytes());
    raw[8..10].copy_from_slice(&0xAAAAu16.to_le_bytes());
    raw[10] = 0x11;
    raw
}

#[test]
fn detects_vms_backup() {
    let entry = read_tap_entry(&make_vms_block()).expect("parse");
    assert!(matches!(
        entry.kind,
        TapDataKind::VmsBlock(BackupBlock { .. })
    ));
    assert_eq!(entry.detected_format, DetectedFormat::VmsBackup);
    write_output("tap", "vms_detect.txt", &format!("{entry:?}"));
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
    write_output(
        "tap",
        "format_detect.txt",
        &format!(
            "RSX={:?} RT11={:?} RSTS={:?}",
            rsx_entry, rt11_entry, rsts_entry
        ),
    );
}

#[test]
fn falls_back_to_raw() {
    let data = vec![1u8, 2, 3, 4];
    let entry = read_tap_entry(&data).unwrap();
    assert_eq!(entry.detected_format, DetectedFormat::Raw);
    write_output("tap", "raw_detect.txt", &format!("{entry:?}"));
}

#[test]
fn non_vms_real_tapes_do_not_panic_or_parse_as_vms() {
    let rsx_data = load_tap_fixture("TA0113.TAP");
    let rsx_entries = read_tap_with_chunks(&rsx_data, 512).unwrap();
    assert!(
        rsx_entries
            .iter()
            .all(|e| e.detected_format != DetectedFormat::VmsBackup),
        "RSX tape should not be treated as VMS BACKUP"
    );

    let rsts_data = load_tap_fixture("TA0013.TAP");
    let rsts_entries = read_tap_with_chunks(&rsts_data, 512).unwrap();
    assert!(
        rsts_entries
            .iter()
            .all(|e| e.detected_format != DetectedFormat::VmsBackup),
        "RSTS/E tape should not be treated as VMS BACKUP"
    );
}
