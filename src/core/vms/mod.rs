pub mod block;
pub mod directory;
pub mod extract;
pub mod file;
pub mod header;
pub mod reconstruct;

use std::collections::BTreeMap;

use crate::core::block::{BlockClassification, TapeBlock};

pub use block::*;
pub use directory::*;
pub use extract::*;
pub use file::*;
pub use header::*;
pub use reconstruct::*;

/// A per-file temporary structure before final reconstruction.
#[derive(Debug, Clone, Default)]
pub struct VmsCollected {
    pub fh2: Option<block::Fh2Record>,
    pub xh2: Option<block::Xh2Record>,
    pub xh3: Option<block::Xh3Record>,
    pub vbn: Vec<block::VbnRecord>,
}

impl VmsCollected {
    pub fn new() -> Self {
        Self {
            fh2: None,
            xh2: None,
            xh3: None,
            vbn: Vec::new(),
        }
    }
}

/// Scan a list of TapeBlocks and collect all VMS structures.
pub fn collect_vms_blocks(blocks: &[TapeBlock]) -> BTreeMap<u32, VmsCollected> {
    let mut out: BTreeMap<u32, VmsCollected> = BTreeMap::new();

    for blk in blocks {
        if let BlockClassification::Vms(_) = &blk.classification {
            if let Some(parsed) = block::classify_vms_block(blk.raw.as_ref(), blk.index) {
                match parsed.kind {
                    block::VmsBlockKind::FileHeader(fh2) => {
                        let id = blk.index;
                        out.entry(id).or_default().fh2 = Some(fh2);
                    }
                    block::VmsBlockKind::Extension(xh2) => {
                        // Attach XH2 to the last FH2 block encountered.
                        if let Some(k) = last_key(&out) {
                            if let Some(entry) = out.get_mut(&k) {
                                entry.xh2 = Some(xh2);
                            }
                        }
                    }
                    block::VmsBlockKind::Ident(xh3) => {
                        if let Some(k) = last_key(&out) {
                            if let Some(entry) = out.get_mut(&k) {
                                entry.xh3 = Some(xh3);
                            }
                        }
                    }
                    block::VmsBlockKind::FileData(vbn) => {
                        if let Some(k) = last_key(&out) {
                            if let Some(entry) = out.get_mut(&k) {
                                entry.vbn.push(vbn);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

fn last_key(map: &BTreeMap<u32, VmsCollected>) -> Option<u32> {
    map.keys().rev().next().cloned()
}
