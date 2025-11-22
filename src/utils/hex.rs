/// Format a slice of bytes with paired hex and ASCII columns.
///
/// Printable ASCII is 0x20..=0x7E; everything else renders as `.`.
pub fn format_hex(data: &[u8]) -> String {
    let mut out = String::new();
    for (row, chunk) in data.chunks(16).enumerate() {
        let offset = row * 16;
        out.push_str(&format!("{:04X}:  ", offset));

        for (idx, byte) in chunk.iter().enumerate() {
            out.push_str(&format!("{:02X} ", byte));
            if idx == 7 {
                out.push(' ');
            }
        }

        // Pad hex column for short final rows to keep ASCII aligned.
        let missing = 16usize.saturating_sub(chunk.len());
        if missing > 0 {
            for _ in 0..missing {
                out.push_str("   ");
            }
            if chunk.len() <= 8 {
                out.push(' ');
            }
        }

        out.push_str(" |");
        for byte in chunk {
            let ch = match byte {
                0x20..=0x7E => *byte as char,
                _ => '.',
            };
            out.push(ch);
        }
        // Pad ASCII column when chunk shorter than 16.
        for _ in chunk.len()..16 {
            out.push('.');
        }
        out.push_str("|\n");
    }
    if !out.is_empty() {
        // Remove trailing newline for cleaner display.
        out.pop();
    }
    out
}
