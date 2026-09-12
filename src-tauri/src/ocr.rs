use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
#[link(name = "Vision", kind = "framework")]
extern "C" {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ModifierFlags {
    pub alt: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub meta: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OcrResult {
    pub text: String,
    pub line_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OcrLine {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(target_os = "macos")]
pub fn get_current_modifier_flags() -> ModifierFlags {
    use core_graphics::event::CGEventFlags;
    use core_graphics::event_source::CGEventSourceStateID;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceFlagsState(state_id: CGEventSourceStateID) -> CGEventFlags;
    }

    let flags = unsafe { CGEventSourceFlagsState(CGEventSourceStateID::CombinedSessionState) };
    ModifierFlags {
        alt: flags.contains(CGEventFlags::CGEventFlagAlternate),
        shift: flags.contains(CGEventFlags::CGEventFlagShift),
        ctrl: flags.contains(CGEventFlags::CGEventFlagControl),
        meta: flags.contains(CGEventFlags::CGEventFlagCommand),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_current_modifier_flags() -> ModifierFlags {
    ModifierFlags {
        alt: false,
        shift: false,
        ctrl: false,
        meta: false,
    }
}

// The actual modifier check at runtime happens in JS (ocrClient.js's
// isOcrModifierActive), against raw ModifierFlags from get_modifier_flags().
// These Rust-side equivalents exist for the unit test coverage the plan
// requires (modifier mapping, invalid modifier) and are not called from a
// command handler.
#[allow(dead_code)]
pub fn map_modifier_name(modifier: &str) -> Option<&'static str> {
    match modifier.trim().to_lowercase().as_str() {
        "option" | "alt" => Some("alt"),
        "shift" => Some("shift"),
        "control" | "ctrl" => Some("ctrl"),
        "command" | "cmd" | "meta" => Some("meta"),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn is_modifier_active(flags: &ModifierFlags, modifier_name: &str) -> bool {
    match map_modifier_name(modifier_name) {
        Some("alt") => flags.alt,
        Some("shift") => flags.shift,
        Some("ctrl") => flags.ctrl,
        Some("meta") => flags.meta,
        _ => false,
    }
}

/// Sort recognized OCR lines by natural reading order:
/// In Apple Vision coordinates, y=0.0 is the bottom and y=1.0 is the top.
/// Lines higher in the image have larger y, so lines must be sorted descending by y.
/// For lines at roughly the same vertical level, they are sorted ascending by x (left to right).
pub fn sort_ocr_lines(mut lines: Vec<OcrLine>) -> Vec<OcrLine> {
    // Phase 1: a plain descending sort by y is a total order (no pairwise
    // threshold), so this step alone can never be non-transitive.
    lines.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));

    // Phase 2: group the now-y-sorted lines into rows using a fixed anchor
    // per row (the row's first line), not a pairwise-recomputed threshold.
    // A pairwise threshold can disagree between adjacent comparisons (e.g.
    // three near-collinear lines where consecutive pairs are "close enough"
    // but the first and last are not), which makes the ordering relation
    // non-transitive and its result undefined. Anchoring to one fixed
    // reference per row avoids that.
    let mut rows: Vec<Vec<OcrLine>> = Vec::new();
    for line in lines {
        let threshold = line.height.max(0.005) * 0.5;
        let starts_new_row = match rows.last() {
            Some(row) => (row[0].y - line.y).abs() > threshold,
            None => true,
        };
        if starts_new_row {
            rows.push(vec![line]);
        } else {
            rows.last_mut().unwrap().push(line);
        }
    }

    // Phase 3: within each row, sort left to right by x.
    let mut result = Vec::with_capacity(rows.iter().map(|row| row.len()).sum());
    for mut row in rows {
        row.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        result.extend(row);
    }
    result
}

pub fn normalize_ocr_text(lines: &[OcrLine]) -> Result<OcrResult, String> {
    let mut normalized_lines = Vec::new();
    for line in lines {
        let trimmed = line.text.trim();
        if !trimmed.is_empty() {
            normalized_lines.push(trimmed.to_string());
        }
    }
    if normalized_lines.is_empty() {
        return Err("No text recognized".to_string());
    }
    let count = normalized_lines.len();
    let text = normalized_lines.join("\n");
    Ok(OcrResult {
        text,
        line_count: count,
    })
}

pub fn validate_image_file_path(path_str: &str) -> Result<std::path::PathBuf, String> {
    let path = std::path::PathBuf::from(path_str);
    if !path.exists() {
        return Err("Image file does not exist".to_string());
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());
    if !matches!(
        ext.as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "tiff" | "tif" | "bmp" | "heic" | "heif")
    ) {
        return Err("Unsupported file type for OCR".to_string());
    }
    let metadata =
        std::fs::metadata(&path).map_err(|e| format!("Failed to inspect image file: {e}"))?;
    if metadata.len() as usize > crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES {
        return Err(format!(
            "Image too large to load: {:.1} MB",
            metadata.len() as f64 / (1024.0 * 1024.0)
        ));
    }
    Ok(path)
}

pub fn is_supported_raster_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    if image::guess_format(bytes).is_ok() {
        return true;
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return true;
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return true;
    }
    false
}

pub fn decode_image_bytes(base64_str: &str) -> Result<Vec<u8>, String> {
    let trimmed = base64_str.trim();
    if trimmed.len() > crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES * 4 / 3 + 4 {
        return Err("Image payload exceeds 20 MB limit".to_string());
    }
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(trimmed)
        .map_err(|e| format!("Invalid base64 payload: {e}"))?;
    if decoded.len() > crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES {
        return Err("Image payload exceeds 20 MB limit".to_string());
    }
    if !is_supported_raster_bytes(&decoded) {
        return Err("Unsupported image format for OCR".to_string());
    }
    Ok(decoded)
}

#[cfg(target_os = "macos")]
pub fn read_clipboard_image_bytes() -> Result<Vec<u8>, String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let pool: id = msg_send![class!(NSAutoreleasePool), new];
        let pboard: id = msg_send![class!(NSPasteboard), generalPasteboard];

        // Try formats in order (TIFF first: the standard macOS screenshot
        // format). If a representation exists but exceeds the size limit,
        // fall through to the next, smaller representation instead of
        // failing immediately -- the pasteboard commonly carries more than
        // one representation of the same image.
        let uti_types = ["public.tiff", "public.png", "public.jpeg"];
        let mut found_oversized = false;

        for uti in uti_types {
            let type_str = NSString::alloc(nil).init_str(uti);
            let data: id = msg_send![pboard, dataForType: type_str];
            let _: () = msg_send![type_str, release];

            if data == nil {
                continue;
            }

            let len: usize = msg_send![data, length];
            let bytes_ptr: *const u8 = msg_send![data, bytes];

            if len == 0 || bytes_ptr.is_null() {
                continue;
            }

            if len > crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES {
                found_oversized = true;
                continue;
            }

            let bytes = std::slice::from_raw_parts(bytes_ptr, len).to_vec();
            let _: () = msg_send![pool, drain];
            return Ok(bytes);
        }

        let file_url_type = NSString::alloc(nil).init_str("public.file-url");
        let file_url_data: id = msg_send![pboard, dataForType: file_url_type];
        let _: () = msg_send![file_url_type, release];

        if file_url_data != nil {
            let ns_url_cls = class!(NSURL);
            let url: id = msg_send![ns_url_cls, URLFromPasteboard: pboard];
            if url != nil {
                let path: id = msg_send![url, path];
                if path != nil {
                    let path_bytes: *const std::os::raw::c_char = msg_send![path, UTF8String];
                    if !path_bytes.is_null() {
                        let path_str = std::ffi::CStr::from_ptr(path_bytes)
                            .to_string_lossy()
                            .to_string();
                        if let Ok(valid_path) = validate_image_file_path(&path_str) {
                            if let Ok(bytes) = std::fs::read(valid_path) {
                                let _: () = msg_send![pool, drain];
                                return Ok(bytes);
                            }
                        }
                    }
                }
            }
        }

        let _: () = msg_send![pool, drain];
        if found_oversized {
            Err("Clipboard image exceeds 20 MB limit".to_string())
        } else {
            Err("No image in clipboard".to_string())
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn read_clipboard_image_bytes() -> Result<Vec<u8>, String> {
    Err("Clipboard OCR is only supported on macOS".to_string())
}

#[cfg(target_os = "macos")]
pub fn recognize_text_from_bytes(bytes: &[u8]) -> Result<OcrResult, String> {
    use cocoa::base::{id, nil, BOOL, YES};
    use cocoa::foundation::NSRect;
    use objc::runtime::Class;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let pool: id = msg_send![class!(NSAutoreleasePool), new];

        let req_handler_cls = match Class::get("VNImageRequestHandler") {
            Some(cls) => cls,
            None => {
                let _: () = msg_send![pool, drain];
                return Err("Vision framework not available (VNImageRequestHandler)".to_string());
            }
        };

        let recognize_cls = match Class::get("VNRecognizeTextRequest") {
            Some(cls) => cls,
            None => {
                let _: () = msg_send![pool, drain];
                return Err("Vision framework not available (VNRecognizeTextRequest)".to_string());
            }
        };

        let ns_data: id =
            msg_send![class!(NSData), dataWithBytes:bytes.as_ptr() length:bytes.len()];
        if ns_data == nil {
            let _: () = msg_send![pool, drain];
            return Err("Failed to create NSData from image bytes".to_string());
        }

        let handler: id = msg_send![req_handler_cls, alloc];
        let handler: id = msg_send![handler, initWithData:ns_data options:nil];
        if handler == nil {
            let _: () = msg_send![pool, drain];
            return Err("Failed to initialize VNImageRequestHandler".to_string());
        }

        let request: id = msg_send![recognize_cls, alloc];
        let request: id = msg_send![request, init];
        if request == nil {
            let _: () = msg_send![handler, release];
            let _: () = msg_send![pool, drain];
            return Err("Failed to initialize VNRecognizeTextRequest".to_string());
        }

        // Recognition configuration
        let _: () = msg_send![request, setRecognitionLevel: 0isize];
        let _: () = msg_send![request, setUsesLanguageCorrection: YES];

        let req_array: id = msg_send![class!(NSArray), arrayWithObject: request];
        let mut error: id = nil;
        let success: BOOL = msg_send![handler, performRequests:req_array error:&mut error];

        if success != YES {
            let error_desc = if error != nil {
                let desc: id = msg_send![error, localizedDescription];
                let c_str: *const std::os::raw::c_char = msg_send![desc, UTF8String];
                if !c_str.is_null() {
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned()
                } else {
                    "Unknown Vision error".to_string()
                }
            } else {
                "Vision request failed".to_string()
            };
            let _: () = msg_send![request, release];
            let _: () = msg_send![handler, release];
            let _: () = msg_send![pool, drain];
            return Err(format!("Vision recognition error: {error_desc}"));
        }

        let results: id = msg_send![request, results];
        let count: usize = if results != nil {
            msg_send![results, count]
        } else {
            0
        };

        if count == 0 {
            let _: () = msg_send![request, release];
            let _: () = msg_send![handler, release];
            let _: () = msg_send![pool, drain];
            return Err("No text recognized".to_string());
        }

        let mut lines = Vec::with_capacity(count);
        for i in 0..count {
            let obs: id = msg_send![results, objectAtIndex: i];
            let bbox: NSRect = msg_send![obs, boundingBox];
            let candidates: id = msg_send![obs, topCandidates: 1usize];
            let cand_count: usize = if candidates != nil {
                msg_send![candidates, count]
            } else {
                0
            };
            if cand_count > 0 {
                let cand: id = msg_send![candidates, objectAtIndex: 0usize];
                let ns_str: id = msg_send![cand, string];
                if ns_str != nil {
                    let c_str: *const std::os::raw::c_char = msg_send![ns_str, UTF8String];
                    if !c_str.is_null() {
                        let text = std::ffi::CStr::from_ptr(c_str)
                            .to_string_lossy()
                            .into_owned();
                        lines.push(OcrLine {
                            text,
                            x: bbox.origin.x,
                            y: bbox.origin.y,
                            width: bbox.size.width,
                            height: bbox.size.height,
                        });
                    }
                }
            }
        }

        let _: () = msg_send![request, release];
        let _: () = msg_send![handler, release];
        let _: () = msg_send![pool, drain];

        let sorted = sort_ocr_lines(lines);
        let result = normalize_ocr_text(&sorted)?;
        log::info!(
            "OCR recognition completed (lines={}, chars={})",
            result.line_count,
            result.text.len()
        );
        Ok(result)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn recognize_text_from_bytes(_bytes: &[u8]) -> Result<OcrResult, String> {
    Err("OCR is only supported on macOS".to_string())
}

pub fn ocr_clipboard() -> Result<OcrResult, String> {
    let bytes = read_clipboard_image_bytes()?;
    recognize_text_from_bytes(&bytes)
}

pub fn ocr_file(path: &str) -> Result<OcrResult, String> {
    let validated_path = validate_image_file_path(path)?;
    let bytes =
        std::fs::read(&validated_path).map_err(|e| format!("Failed to read image file: {e}"))?;
    recognize_text_from_bytes(&bytes)
}

pub fn ocr_bytes(bytes_base64: &str) -> Result<OcrResult, String> {
    let bytes = decode_image_bytes(bytes_base64)?;
    recognize_text_from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_normalization() {
        let lines = vec![
            OcrLine {
                text: "  Hello World  ".to_string(),
                x: 0.1,
                y: 0.8,
                width: 0.5,
                height: 0.05,
            },
            OcrLine {
                text: "   ".to_string(),
                x: 0.1,
                y: 0.7,
                width: 0.5,
                height: 0.05,
            },
            OcrLine {
                text: "Second Line".to_string(),
                x: 0.1,
                y: 0.6,
                width: 0.5,
                height: 0.05,
            },
        ];

        let result = normalize_ocr_text(&lines).expect("should normalize text");
        assert_eq!(result.text, "Hello World\nSecond Line");
        assert_eq!(result.line_count, 2);
    }

    #[test]
    fn test_empty_ocr_result() {
        let empty_lines: Vec<OcrLine> = vec![];
        assert_eq!(
            normalize_ocr_text(&empty_lines),
            Err("No text recognized".to_string())
        );

        let whitespace_lines = vec![OcrLine {
            text: "   \n  \t ".to_string(),
            x: 0.1,
            y: 0.8,
            width: 0.5,
            height: 0.05,
        }];
        assert_eq!(
            normalize_ocr_text(&whitespace_lines),
            Err("No text recognized".to_string())
        );
    }

    #[test]
    fn test_sorting_order_of_recognized_lines() {
        // Line 1 is at top of image (y=0.9)
        // Line 2 is in middle (y=0.5)
        // Line 3 is at bottom (y=0.1)
        // Two words on middle line: left (x=0.1) and right (x=0.5)
        let lines = vec![
            OcrLine {
                text: "Bottom".to_string(),
                x: 0.1,
                y: 0.1,
                width: 0.2,
                height: 0.05,
            },
            OcrLine {
                text: "Middle Right".to_string(),
                x: 0.5,
                y: 0.5,
                width: 0.2,
                height: 0.05,
            },
            OcrLine {
                text: "Top".to_string(),
                x: 0.1,
                y: 0.9,
                width: 0.2,
                height: 0.05,
            },
            OcrLine {
                text: "Middle Left".to_string(),
                x: 0.1,
                y: 0.5,
                width: 0.2,
                height: 0.05,
            },
        ];

        let sorted = sort_ocr_lines(lines);
        let texts: Vec<String> = sorted.into_iter().map(|l| l.text).collect();
        assert_eq!(texts, vec!["Top", "Middle Left", "Middle Right", "Bottom"]);
    }

    #[test]
    fn test_modifier_mapping() {
        assert_eq!(map_modifier_name("Option"), Some("alt"));
        assert_eq!(map_modifier_name("alt"), Some("alt"));
        assert_eq!(map_modifier_name("Shift"), Some("shift"));
        assert_eq!(map_modifier_name("Control"), Some("ctrl"));
        assert_eq!(map_modifier_name("ctrl"), Some("ctrl"));
        assert_eq!(map_modifier_name("Command"), Some("meta"));
        assert_eq!(map_modifier_name("cmd"), Some("meta"));
        assert_eq!(map_modifier_name("meta"), Some("meta"));
        assert_eq!(map_modifier_name("None"), None);
        assert_eq!(map_modifier_name("invalid"), None);

        let flags = ModifierFlags {
            alt: true,
            shift: false,
            ctrl: false,
            meta: true,
        };
        assert!(is_modifier_active(&flags, "Option"));
        assert!(is_modifier_active(&flags, "Command"));
        assert!(!is_modifier_active(&flags, "Shift"));
        assert!(!is_modifier_active(&flags, "Control"));
    }

    #[test]
    fn test_payload_limit_base64() {
        // Exceeding MAX_IMAGE_PAYLOAD_BYTES (20MB)
        let large_len = crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES * 4 / 3 + 100;
        let large_b64 = "A".repeat(large_len);
        let err = decode_image_bytes(&large_b64);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("20 MB limit"));
    }

    #[test]
    fn test_unsupported_input() {
        // Invalid base64
        let err = decode_image_bytes("Not base 64!!");
        assert!(err.is_err());

        // Valid base64 but not image bytes
        use base64::Engine;
        let non_image =
            base64::engine::general_purpose::STANDARD.encode(b"This is just plain text");
        let err2 = decode_image_bytes(&non_image);
        assert!(err2.is_err());
        assert!(err2.unwrap_err().contains("Unsupported image format"));

        // Unsupported file extension
        let err3 = validate_image_file_path("test.txt");
        assert!(err3.is_err());
    }

    #[test]
    fn test_payload_limit_file() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("collector_ocr_test_oversize.png");
        let file = std::fs::File::create(&temp_file).expect("create temp file");
        file.set_len((crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES + 1024) as u64)
            .expect("set file length");
        drop(file);

        let result = validate_image_file_path(&temp_file.to_string_lossy());
        let _ = std::fs::remove_file(&temp_file);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Image too large"));
    }
}
