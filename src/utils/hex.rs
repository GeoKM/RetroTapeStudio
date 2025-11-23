/// Legacy hex formatter kept for compatibility.
pub fn format_hex(data: &[u8]) -> String {
    format_hex_with_ascii(data)
}

/// Format bytes with address, grouped hex, and ASCII columns.
pub fn format_hex_with_ascii(data: &[u8]) -> String {
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

        let missing = 16usize.saturating_sub(chunk.len());
        if missing > 0 {
            for pad_idx in 0..missing {
                if chunk.len() + pad_idx == 8 {
                    out.push(' ');
                }
                out.push_str("   ");
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
        for _ in 0..missing {
            out.push(' ');
        }
        out.push_str("|\n");
    }
    if !out.is_empty() {
        out.pop();
    }
    out
}
