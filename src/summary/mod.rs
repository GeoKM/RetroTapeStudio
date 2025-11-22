//! Save-set summary computation: aggregates counts, histograms, efficiency, and log metadata for the UI.
use std::collections::HashMap;

use crate::backup::vms::{format_protection, RecordFormat};
use crate::gui::state::AppState;
use crate::log::parse::{LogData, LogLevel};
use crate::utils::text::sanitize_display;

#[derive(Debug, Clone, PartialEq)]
pub struct SaveSetSummary {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_blocks: usize,
    pub total_bytes: usize,
    pub largest_file: Option<String>,
    pub smallest_file: Option<String>,
    pub rfm_hist: HashMap<String, usize>,
    pub protection_hist: HashMap<String, usize>,
    pub block_efficiency: f64,
    pub log_warnings: usize,
    pub log_errors: usize,
    pub tracks: Option<String>,
    pub density: Option<String>,
    pub blocks_read: Option<String>,
}

pub fn compute_saveset_summary(state: &AppState) -> SaveSetSummary {
    let total_files = state.vms_files.len();
    let total_directories = state
        .vms_fs
        .as_ref()
        .map(|fs| count_dirs(&fs.root))
        .unwrap_or(0);

    let mut total_blocks = 0usize;
    let mut total_bytes = 0usize;
    let mut rfm_hist: HashMap<String, usize> = HashMap::new();
    let mut protection_hist: HashMap<String, usize> = HashMap::new();

    let mut largest: Option<(String, usize)> = None;
    let mut smallest: Option<(String, usize)> = None;

    let mut block_size_sum = 0usize;

    for file in &state.vms_files {
        let payload_size: usize = file.blocks.iter().map(|b| b.payload.len()).sum();
        let block_size: usize = file.blocks.iter().map(|b| b.block_size as usize).sum();
        total_bytes += payload_size;
        total_blocks += file.blocks.len();
        block_size_sum += block_size;

        let name = format!(
            "{};{} [UIC {:X}]",
            file.path, file.headers.version, file.headers.owner_uic
        );
        match largest {
            Some((_, size)) if size >= payload_size => {}
            _ => largest = Some((sanitize_display(&name), payload_size)),
        }
        match smallest {
            Some((_, size)) if size <= payload_size => {}
            _ => smallest = Some((sanitize_display(&name), payload_size)),
        }

        let rfm_key = record_format_text(&file.headers.record_format).to_string();
        *rfm_hist.entry(rfm_key).or_insert(0) += 1;

        let prot = format_protection(file.headers.protection_mask);
        *protection_hist.entry(prot).or_insert(0) += 1;
    }

    let block_efficiency = if block_size_sum == 0 {
        0.0
    } else {
        total_bytes as f64 / block_size_sum as f64
    };

    let (log_warnings, log_errors, tracks, density, blocks_read) = state
        .log_state
        .data
        .as_ref()
        .map(log_stats)
        .unwrap_or((0, 0, None, None, None));

    SaveSetSummary {
        total_files,
        total_directories,
        total_blocks,
        total_bytes,
        largest_file: largest.map(|(n, s)| format!("{} ({} bytes)", n, s)),
        smallest_file: smallest.map(|(n, s)| format!("{} ({} bytes)", n, s)),
        rfm_hist,
        protection_hist,
        block_efficiency,
        log_warnings,
        log_errors,
        tracks,
        density,
        blocks_read,
    }
}

fn record_format_text(rfm: &RecordFormat) -> &'static str {
    match rfm {
        RecordFormat::Udf => "UDF",
        RecordFormat::Vfc => "VFC",
        RecordFormat::Var => "VAR",
        RecordFormat::Fix => "FIX",
        RecordFormat::Unknown(_) => "UNKNOWN",
    }
}

fn count_dirs(node: &crate::backup::extract::DirectoryNode) -> usize {
    let mut count = node.children.len();
    for child in &node.children {
        count += count_dirs(child);
    }
    count
}

fn log_stats(log: &LogData) -> (usize, usize, Option<String>, Option<String>, Option<String>) {
    let mut warnings = 0;
    let mut errors = 0;
    for entry in &log.entries {
        match entry.level {
            LogLevel::Warning => warnings += 1,
            LogLevel::Error => errors += 1,
            LogLevel::Info => {}
        }
    }
    let tracks = log
        .metadata
        .get("Tracks")
        .map(|s| sanitize_display(s));
    let density = log
        .metadata
        .get("Density")
        .map(|s| sanitize_display(s));
    let blocks_read = log
        .metadata
        .get("Blocks read")
        .map(|s| sanitize_display(s));
    (warnings, errors, tracks, density, blocks_read)
}
