pub(super) fn default_autostart_enabled() -> bool {
    false
}
pub(super) fn default_edge_enabled() -> bool {
    true
}

pub(super) fn default_true() -> bool {
    true
}

pub(super) fn default_daily_note_insert_position() -> String {
    "bottom".to_string()
}

pub(super) fn default_daily_note_create_timeout_ms() -> u64 {
    15000
}

pub(super) fn default_false() -> bool {
    false
}

pub(super) fn default_reaction_time_ms() -> u64 {
    50
}

pub(super) fn default_edge_open_delay_ms() -> u64 {
    1000
}

pub(super) fn default_reader_edge_side() -> String {
    "left".to_string()
}

pub(super) fn default_vault_path() -> String {
    dirs::home_dir()
        .map(|h| h.join("Vault").to_string_lossy().to_string())
        .unwrap_or_else(|| String::new())
}

pub(super) fn default_screenshot_path() -> String {
    "Grafiken/Screenshots".to_string()
}

pub(super) fn default_notes_folder() -> String {
    "Notes".to_string()
}

pub(super) fn default_save_to_daily_shortcut() -> String {
    "Cmd+Enter".to_string()
}

pub(super) fn default_save_as_note_shortcut() -> String {
    "Cmd+Shift+Enter".to_string()
}

pub(super) fn default_append_to_note_shortcut() -> String {
    "Cmd+Option+Enter".to_string()
}

pub(super) fn default_window_width() -> u32 {
    350
}

pub(super) fn default_window_height() -> u32 {
    600
}

pub(super) fn default_reader_shortcut() -> String {
    "Cmd+Shift+R".to_string()
}

pub(super) fn default_reader_open_in_obsidian_shortcut() -> String {
    "Cmd+Shift+O".to_string()
}

pub(super) fn default_reader_command_palette_shortcut() -> String {
    "Cmd+P".to_string()
}

pub(super) fn default_reader_next_tab_shortcut() -> String {
    "Option+Right".to_string()
}

pub(super) fn default_reader_previous_tab_shortcut() -> String {
    "Option+Left".to_string()
}

pub(super) fn default_note_filename_template() -> String {
    "note-YYYY-MM-DD-HHmmss".to_string()
}

pub(super) fn default_note_template() -> String {
    "---\ncreated: <% tp.date.now(\"YYYY-MM-DD hh:mm\") %>\nmodified: \ndaily: \"[[<% tp.date.now(\"YYYY-MM-DD\") %>]]\"\ntags: inbox\ntype: inbox\n---".to_string()
}

pub(super) fn default_image_width() -> String {
    "600".to_string()
}

pub(super) fn default_autocomplete_results() -> u32 {
    20
}

pub(super) fn default_overlay_strength() -> u32 {
    10
}

pub(super) fn default_window_blur() -> u32 {
    55
}

pub(super) fn default_window_saturation() -> u32 {
    85
}

pub(super) fn default_window_brightness() -> i32 {
    -85
}

pub(super) fn default_reader_width() -> u32 {
    400
}

pub(super) fn default_reader_height() -> u32 {
    800
}

pub(super) fn default_text_color() -> String {
    "#ffffff".to_string()
}

pub(super) fn default_accent_color() -> String {
    "#8b5cf6".to_string()
}

pub(super) fn default_internal_link_color() -> String {
    "#a78bfa".to_string()
}

pub(super) fn default_external_link_color() -> String {
    "#60a5fa".to_string()
}

pub(super) fn default_capture_text_shortcut() -> String {
    "Cmd+Shift+C".to_string()
}

pub(super) fn default_daily_note_folder() -> String {
    String::new()
}

pub(super) fn default_daily_note_format() -> String {
    "YYYY-MM-DD".to_string()
}

pub(super) fn default_ocr_shortcut() -> String {
    "Cmd+Shift+T".to_string()
}

pub(super) fn default_ocr_drag_modifier() -> String {
    "Option".to_string()
}

