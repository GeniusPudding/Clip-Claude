use std::path::Path;

pub fn format_payload(path: &Path, width: u32, height: u32) -> String {
    format!(
        "[clipbridge] Pasted image ({width}x{height})\n\
         File: {}\n\
         Please open and analyze this file using your image-reading tool.\n\
         (This text was auto-injected because the terminal cannot display images directly.)",
        path.display()
    )
}
