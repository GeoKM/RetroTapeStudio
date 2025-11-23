use super::*;
use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile, TapePath};

pub fn reconstruct_vms(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let mut results = Vec::new();
    let mut current: Option<(VmsFileHeader, Vec<u32>)> = None;

    for blk in blocks {
        let Some(vms) = (match &blk.classification {
            crate::core::block::BlockClassification::Vms(_) => {
                super::block::classify_vms_block(&blk.raw, blk.index)
            }
            _ => None,
        }) else {
            continue;
        };

        match vms.kind {
            VmsBlockKind::Header => {
                if let Some((header, block_list)) = current.take() {
                    results.push(make_file(header, block_list, blocks));
                }
                let hdr = VmsFileHeader::parse(&vms.raw).unwrap_or(VmsFileHeader {
                    filename: "UNKNOWN".into(),
                    version: 0,
                    uic: (0, 0),
                    record_format: 0,
                    record_attributes: 0,
                    block_count: 0,
                });
                current = Some((hdr, vec![blk.index]));
            }

            VmsBlockKind::FileData | VmsBlockKind::Continuation => {
                if let Some((_hdr, list)) = &mut current {
                    list.push(blk.index);
                }
            }

            VmsBlockKind::Trailer => {
                if let Some((header, block_list)) = current.take() {
                    results.push(make_file(header, block_list, blocks));
                }
                break;
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
