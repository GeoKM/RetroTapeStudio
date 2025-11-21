/// Format a slice of bytes as a simple hex dump string.
pub fn format_hex(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, byte) in data.iter().enumerate() {
        if i % 16 == 0 {
            if i != 0 {
                out.push('\n');
            }
            out.push_str(&format!("{:04X}: ", i));
        }
        out.push_str(&format!("{:02X} ", byte));
    }
    out
}
