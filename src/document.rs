use std::{fs, io, path::{Path, PathBuf}};

const TEMPLATE: &str = "contract Example {\n    case works {\n        given int value = 1;\n        call value++;\n        expect returns 2;\n    }\n}\n";

pub struct Document {
    pub path: Option<PathBuf>,
    pub text: String,
    saved_text: String,
    pub revision: u64,
    pub line_ending: &'static str,
}

impl Default for Document { fn default() -> Self { Self::new() } }

impl Document {
    pub fn new() -> Self { Self { path: None, text: TEMPLATE.into(), saved_text: TEMPLATE.into(), revision: 0, line_ending: "LF" } }
    pub fn title(&self) -> String { self.path.as_deref().and_then(Path::file_name).map(|v| v.to_string_lossy().into_owned()).unwrap_or_else(|| "Untitled.cxxp".into()) }
    pub fn is_dirty(&self) -> bool { self.text != self.saved_text }
    pub fn changed(&mut self) { self.revision = self.revision.wrapping_add(1); }
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let text = fs::read_to_string(&path)?;
        let line_ending = if text.contains("\r\n") { "CRLF" } else { "LF" };
        Ok(Self { path: Some(path), saved_text: text.clone(), text, revision: 0, line_ending })
    }
    pub fn save_to(&mut self, path: PathBuf) -> io::Result<()> { fs::write(&path, self.text.as_bytes())?; self.path = Some(path); self.saved_text.clone_from(&self.text); Ok(()) }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn dirty_tracks_saved_content() { let mut d=Document::new(); assert!(!d.is_dirty()); d.text.push('x'); assert!(d.is_dirty()); }
    #[test] fn revision_advances() { let mut d=Document::new(); d.changed(); assert_eq!(d.revision,1); }
}
