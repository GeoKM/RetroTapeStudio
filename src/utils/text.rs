/// Sanitize text for UI display: keep ASCII printable characters and spaces, and
/// replace anything else (including control/non-ASCII) with two dots.
pub fn sanitize_display(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if (ch.is_ascii_graphic() || ch == ' ') && !ch.is_control() {
            out.push(ch);
        } else {
            out.push_str("..");
        }
    }
    out
}

/// Lightweight check for printable ASCII without revealing contents.
pub fn is_mostly_text(data: &[u8]) -> bool {
    let printable = data
        .iter()
        .all(|b| matches!(b, b'\n' | b'\r' | b'\t' | 0x20..=0x7E));
    printable && !data.is_empty()
}
