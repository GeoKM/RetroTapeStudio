use super::*;
use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile, TapePath};

pub fn reconstruct_vms(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let mut results = Vec::new();
    let mut current: Option<(VmsFileHeader, Vec<u32>)> = None;

    for blk in blocks {
        let Some(vms) = (match &blk.classification {
            crate::core::block::BlockClassification::Vms(_) => {
                super::block::classify_vms_block(blk.raw.as_ref(), blk.index)
            }
            _ => None,
        }) else {
            continue;
        };

        match vms.kind {
            VmsBlockKind::FileHeader(hdr) => {
                if let Some((header, block_list)) = current.take() {
                    results.push(make_file(header, block_list, blocks));
                }
                let filename = if hdr.file_type.is_empty() {
                    hdr.file_name.clone()
                } else {
                    format!("{}.{}", hdr.file_name, hdr.file_type)
                };
                let header = VmsFileHeader {
                    filename,
                    version: hdr.version,
                    uic: hdr.uic,
                    record_format: hdr.record_format,
                    record_attributes: u16::from(hdr.record_attributes),
                    block_count: hdr.block_count,
                };
                current = Some((header, vec![blk.index]));
            }

            VmsBlockKind::FileData(_) => {
                if let Some((_hdr, list)) = &mut current {
                    list.push(blk.index);
                }
            }

            _ => {}
        }
    }

    if let Some((header, block_list)) = current.take() {
        results.push(make_file(header, block_list, blocks));
    }

    results
}

fn make_file(header: VmsFileHeader, block_list: Vec<u32>, blocks: &[TapeBlock]) -> TapeFile {
    let mut data = Vec::new();
    for idx in &block_list {
        if let Some(b) = blocks.iter().find(|x| x.index == *idx) {
            data.extend_from_slice(&b.raw);
        }
    }

    TapeFile {
        path: TapePath::new(vec![header.filename.clone()]),
        format: crate::core::block::TapeFormat::Vms,
        size_bytes: data.len() as u64,
        blocks: block_list,
        metadata: FileMetadata::Vms(crate::core::file::VmsFileMetadata { placeholder: false }),
        children: vec![],
    }
}
