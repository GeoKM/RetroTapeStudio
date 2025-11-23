use crate::core::block::{TapeBlock, TapeFormat};
use crate::core::file::{
    FileMetadata, RstsFileMetadata, RsxFileMetadata, Rt11FileMetadata, TapeFile, TapePath,
};
use crate::core::parse::{parse_classified_block, ParsedBlock};

pub fn reconstruct_rsx(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let mut tree = Vec::new();
    for block in blocks {
        match parse_classified_block(block) {
            Some(ParsedBlock::Rsx(info)) => {
                let mut path = vec![format!("[{:o},{:o}]", info.uic.0, info.uic.1)];
                if info.name.is_empty() {
                    path.push(format!("unnamed_{:05}", block.index));
                } else {
                    path.push(info.name.clone());
                }
                let metadata = FileMetadata::Rsx(RsxFileMetadata {
                    uic: info.uic,
                    protection: info.protection,
                    is_directory: info.is_directory,
                });
                insert_into_tree(
                    &mut tree,
                    TapeFile {
                        format: TapeFormat::Rsx,
                        path: TapePath::new(path),
                        size_bytes: block.size as u64,
                        blocks: vec![block.index],
                        metadata,
                        children: Vec::new(),
                    },
                );
            }
            _ => insert_into_tree(&mut tree, raw_block_entry(block)),
        }
    }
    tree
}

pub fn reconstruct_rt11(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let mut tree = Vec::new();
    for block in blocks {
        match parse_classified_block(block) {
            Some(ParsedBlock::Rt11(info)) if !info.entries.is_empty() => {
                let dir = format!("rt11_dir_{:05}", block.index);
                for entry in info.entries {
                    let name = format_rt11_name(&entry.name, &entry.ext);
                    let path = TapePath::new(vec![dir.clone(), name]);
                    let estimated_bytes = u64::from(entry.length_blocks).saturating_mul(512);
                    let size_bytes = estimated_bytes.max(block.size as u64);
                    let metadata = FileMetadata::Rt11(Rt11FileMetadata {
                        start_block: entry.start_block,
                        length_blocks: entry.length_blocks,
                        ext: entry.ext,
                    });
                    insert_into_tree(
                        &mut tree,
                        TapeFile {
                            format: TapeFormat::Rt11,
                            path,
                            size_bytes,
                            blocks: vec![block.index],
                            metadata,
                            children: Vec::new(),
                        },
                    );
                }
            }
            _ => insert_into_tree(&mut tree, raw_block_entry(block)),
        }
    }
    tree
}

pub fn reconstruct_rsts(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let mut tree = Vec::new();
    for block in blocks {
        match parse_classified_block(block) {
            Some(ParsedBlock::Rsts(info)) if !info.entries.is_empty() => {
                for entry in info.entries {
                    let mut path = Vec::new();
                    path.push(format!("[{:o},{:o}]", entry.owner_uic.0, entry.owner_uic.1));
                    if entry.name.is_empty() {
                        path.push(format!("entry_{:05}", block.index));
                    } else {
                        path.push(entry.name.clone());
                    }
                    let metadata = FileMetadata::Rsts(RstsFileMetadata {
                        owner_uic: entry.owner_uic,
                        blocks: entry.blocks,
                        status: entry.status,
                    });
                    let size_bytes = entry.blocks as u64 * 512;
                    insert_into_tree(
                        &mut tree,
                        TapeFile {
                            format: TapeFormat::Rsts,
                            path: TapePath::new(path),
                            size_bytes,
                            blocks: vec![block.index],
                            metadata,
                            children: Vec::new(),
                        },
                    );
                }
            }
            _ => insert_into_tree(&mut tree, raw_block_entry(block)),
        }
    }
    tree
}

fn raw_block_entry(block: &TapeBlock) -> TapeFile {
    TapeFile {
        format: TapeFormat::Raw,
        path: TapePath::new(vec![format!("block_{:05}", block.index)]),
        size_bytes: block.size as u64,
        blocks: vec![block.index],
        metadata: FileMetadata::Raw,
        children: Vec::new(),
    }
}

fn insert_into_tree(tree: &mut Vec<TapeFile>, file: TapeFile) {
    if file.path.elements.is_empty() {
        return;
    }

    let mut current_level = tree;
    let path = file.path.elements.clone();
    for depth in 0..path.len().saturating_sub(1) {
        let prefix: Vec<String> = path[..=depth].to_vec();
        let next = current_level
            .iter()
            .position(|node| node.path.elements == prefix);
        let idx = match next {
            Some(i) => i,
            None => {
                current_level.push(TapeFile {
                    format: file.format.clone(),
                    path: TapePath::new(prefix.clone()),
                    size_bytes: 0,
                    blocks: Vec::new(),
                    metadata: FileMetadata::Raw,
                    children: Vec::new(),
                });
                current_level.len() - 1
            }
        };
        current_level = &mut current_level[idx].children;
    }

    current_level.push(file);
}

fn format_rt11_name(name: &str, ext: &str) -> String {
    if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    }
}
