use crate::core::block::{
    BlockClassification, RstsBlockKind, RsxBlockKind, Rt11BlockKind, TapeBlock, TapeFormat,
    VmsBlockKind,
};

pub fn detect_block_format(block: &TapeBlock) -> BlockClassification {
    let data = block.raw.as_ref();

    if data.len() >= 20 {
        // VMS backup detection
        let record_type = data[0];
        let class = data[2];

        if record_type == 0x02 && (class == 0x44 || class == 0x45 || class == 0x46) {
            return BlockClassification::Vms(VmsBlockKind::Placeholder);
        }
    }

    // RSX-11M block detection
    if data.len() == 512 {
        if data.starts_with(&[0x31, 0x00]) || data.starts_with(&[0x40, 0x00]) {
            return BlockClassification::Rsx(RsxBlockKind::Placeholder);
        }
    }

    // RT-11 directory block detection
    if data.len() == 512 {
        if data[0] != 0 && data[2] != 0 {
            return BlockClassification::Rt11(Rt11BlockKind::Placeholder);
        }
    }

    // RSTS/E UFD/MFD detection (very loose)
    if data.len() == 512 && data[1] == 0 && data[4] == 0 {
        return BlockClassification::Rsts(RstsBlockKind::Placeholder);
    }

    BlockClassification::Raw
}

pub fn analyze_blocks(blocks: &mut [TapeBlock]) -> TapeFormat {
    let mut vms = 0usize;
    let mut rsx = 0usize;
    let mut rt11 = 0usize;
    let mut rsts = 0usize;

    for blk in blocks.iter_mut() {
        let c = detect_block_format(blk);
        match &c {
            BlockClassification::Vms(_) => vms += 1,
            BlockClassification::Rsx(_) => rsx += 1,
            BlockClassification::Rt11(_) => rt11 += 1,
            BlockClassification::Rsts(_) => rsts += 1,
            _ => {}
        }
        blk.classification = c;
    }

    // Winner selection (simple strongest-match)
    if vms > rsx && vms > rt11 && vms > rsts {
        TapeFormat::Vms
    } else if rsx > rt11 && rsx > rsts {
        TapeFormat::Rsx
    } else if rt11 > rsts {
        TapeFormat::Rt11
    } else if rsts > 0 {
        TapeFormat::Rsts
    } else {
        TapeFormat::Unknown
    }
}
