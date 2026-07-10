use anyhow::Result;
use std::path::Path;

pub struct ExtractedText {
    pub text: String,
    pub word_count: usize,
    #[allow(dead_code)]
    pub char_count: usize,
}

pub fn extract_text(path: &Path, _ocr: bool) -> Result<ExtractedText> {
    let bytes = std::fs::read(path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| anyhow::anyhow!("PDF extraction failed: {}", e))?;

    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();

    Ok(ExtractedText {
        text,
        word_count,
        char_count,
    })
}

pub fn estimate_page_count(path: &Path) -> usize {
    use lopdf::Document;
    Document::load(path)
        .ok()
        .map(|doc| doc.get_pages().len())
        .unwrap_or(0)
}
