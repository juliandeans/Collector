/// Clean up the family list: trim, drop empty and hidden (".Name") families,
/// sort case-insensitively and remove duplicates.
pub(crate) fn normalize_families(families: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = families
        .into_iter()
        .map(|family| family.trim().to_string())
        .filter(|family| !family.is_empty() && !family.starts_with('.'))
        .collect();
    result.sort_by_key(|family| family.to_lowercase());
    result.dedup();
    result
}

#[cfg(target_os = "macos")]
fn read_available_font_families() -> Vec<String> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;
    use std::os::raw::c_char;

    unsafe {
        let pool: id = msg_send![class!(NSAutoreleasePool), new];
        let mut families = Vec::new();

        let manager: id = msg_send![class!(NSFontManager), sharedFontManager];
        if manager != nil {
            let names: id = msg_send![manager, availableFontFamilies];
            if names != nil {
                let count: usize = msg_send![names, count];
                for index in 0..count {
                    let name: id = msg_send![names, objectAtIndex: index];
                    if name == nil {
                        continue;
                    }
                    let utf8: *const c_char = msg_send![name, UTF8String];
                    if !utf8.is_null() {
                        families.push(CStr::from_ptr(utf8).to_string_lossy().into_owned());
                    }
                }
            }
        }

        let _: () = msg_send![pool, drain];
        families
    }
}

#[cfg(not(target_os = "macos"))]
fn read_available_font_families() -> Vec<String> {
    Vec::new()
}

/// Installed font families for the Font Family setting.
/// Sync on purpose: Tauri runs sync commands on the main thread, which AppKit needs.
#[tauri::command]
pub(crate) fn list_font_families() -> Vec<String> {
    normalize_families(read_available_font_families())
}

#[cfg(test)]
mod tests {
    use super::normalize_families;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn sorts_case_insensitively_and_removes_duplicates() {
        let result = normalize_families(strings(&["menlo", "Arial", "Inter", "Arial"]));
        assert_eq!(result, strings(&["Arial", "Inter", "menlo"]));
    }

    #[test]
    fn drops_empty_and_hidden_system_families() {
        let result = normalize_families(strings(&["", "  ", ".AppleSystemUIFont", " Georgia "]));
        assert_eq!(result, strings(&["Georgia"]));
    }
}
