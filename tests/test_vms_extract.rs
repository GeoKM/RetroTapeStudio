use retro_tape_studio_v6_safe::backup::extract::{assemble_vms_files, VmsFileSystem, build_directory_tree};
use retro_tape_studio_v6_safe::backup::vms::BackupBlock;
use retro_tape_studio_v6_safe::tap::reader::{TapDataKind, TapEntry};
use retro_tape_studio_v6_safe::tap::DetectedFormat;
mod common;
use common::write_output;

fn make_block(seq: u32, payload: &[u8]) -> BackupBlock {
    BackupBlock {
        block_size: (10 + payload.len()) as u16,
        format_version: 1,
        phase: 1,
        sequence_number: seq,
        checksum: 0,
        payload: payload.to_vec(),
    }
}

#[test]
fn assembles_files_and_fs() {
    let fh2 = {
        let mut data = Vec::new();
        data.push(0x02);
        let name = "DIR.FILE";
        data.push(name.len() as u8);
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.push(0);
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data
    };

    let entries = vec![
        TapEntry {
            length: 32,
            kind: TapDataKind::VmsBlock(make_block(1, &fh2)),
            log_level: None,
            detected_format: DetectedFormat::VmsBackup,
        },
        TapEntry {
            length: 16,
            kind: TapDataKind::VmsBlock(make_block(2, b"data")),
            log_level: None,
            detected_format: DetectedFormat::VmsBackup,
        },
    ];

    let files = assemble_vms_files(&entries);
    assert_eq!(files.len(), 1);
    assert!(files[0].path.contains("DIR"));
    let fs = build_directory_tree(&files);
    assert!(matches!(fs, VmsFileSystem { .. }));
    write_output(
        "extract",
        "vms_files.txt",
        &format!("files={} path={}", files.len(), files[0].path),
    );
}
