use wasm_bindgen::prelude::*;
use removemetadata_engine as engine;

/// Process a file and return the result as a JS object.
/// Args: name (string), data (Uint8Array), author (string), source (string)
/// Returns: { data: Uint8Array, removed: number, type: string } or null
#[wasm_bindgen]
pub fn process_file(name: &str, data: &[u8], author: &str, source: &str) -> Option<JsValue> {
    let meta = engine::Metadata::new(author, source);
    let result = engine::process_file(name, data, &meta)?;
    let obj = js_sys::Object::new();
    let arr = js_sys::Uint8Array::from(result.output.as_slice());
    js_sys::Reflect::set(&obj, &"data".into(), &arr).ok()?;
    js_sys::Reflect::set(&obj, &"removed".into(), &JsValue::from_f64(result.removed as f64)).ok()?;
    js_sys::Reflect::set(&obj, &"type".into(), &JsValue::from_str(&result.file_type)).ok()?;
    Some(obj.into())
}

/// Process a file with full metadata fields (all 8).
#[wasm_bindgen]
pub fn process_file_meta(
    name: &str, data: &[u8],
    author: &str, source: &str, title: &str, description: &str,
    credit: &str, keywords: &str, category: &str, comments: &str
) -> Option<JsValue> {
    let meta = engine::Metadata {
        author: author.to_string(),
        source: source.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        credit: credit.to_string(),
        keywords: keywords.to_string(),
        category: category.to_string(),
        comments: comments.to_string(),
    };
    let result = engine::process_file(name, data, &meta)?;
    let obj = js_sys::Object::new();
    let arr = js_sys::Uint8Array::from(result.output.as_slice());
    js_sys::Reflect::set(&obj, &"data".into(), &arr).ok()?;
    js_sys::Reflect::set(&obj, &"removed".into(), &JsValue::from_f64(result.removed as f64)).ok()?;
    js_sys::Reflect::set(&obj, &"type".into(), &JsValue::from_str(&result.file_type)).ok()?;
    Some(obj.into())
}

/// Detect file type from name and magic bytes.
#[wasm_bindgen]
pub fn detect_type(name: &str, data: &[u8]) -> String {
    engine::detect_file_type(name, data).to_string()
}

/// Check if a filename is supported.
#[wasm_bindgen]
pub fn is_supported(name: &str) -> bool {
    engine::is_supported(name)
}

/// Clean a filename (remove ChatGPT/DALL-E/Midjourney prefix).
#[wasm_bindgen]
pub fn clean_filename(name: &str) -> String {
    engine::clean_filename(name)
}

/// Get default author constant.
#[wasm_bindgen]
pub fn default_author() -> String {
    engine::DEFAULT_AUTHOR.to_string()
}

/// Get default source constant.
#[wasm_bindgen]
pub fn default_source() -> String {
    engine::DEFAULT_SOURCE.to_string()
}
