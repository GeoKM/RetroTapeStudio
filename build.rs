fn main() {
    for entry in walkdir::WalkDir::new("src") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            if !text.is_ascii() {
                panic!("Non-ASCII detected in {}!", entry.path().display());
            }
        }
    }
}
