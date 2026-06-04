import {
  writable,
  derived
} from 'svelte/store';

export const defaultSettings = {
  vault_name: 'Vault',
  vault_path: '',
  edge_side: 'right',
  window_width: 350,
  window_height: 600,
  reader_width: 400,
  reader_height: 800,
  border_radius: 12,
  overlay_color: '#ffffff',
  font_family: '-apple-system, BlinkMacSystemFont, SF Pro Display',
  font_size: 14,
  daily_note_folder: '',
  daily_note_format: 'YYYY-MM-DD',
  daily_note_path: '',
  screenshot_path: 'Grafiken/Screenshots',
  image_folder: 'Grafiken/Screenshots',
  image_filename: 'screenshot-YYYY-MM-DD-HHmmss',
  default_image_width: '600',
  entry_header: '#### HH:mm',
  daily_note_target_heading: '',
  daily_note_insert_position: 'bottom',
  daily_note_create_heading_if_missing: false,
  daily_note_create_if_missing: false,
  daily_note_create_timeout_ms: 15000,
  show_capture_action_bar: true,
  show_note_paths: true,
  autocomplete_results: 20,
  global_shortcut: 'Cmd+Shift+N',
  global_shortcut_closes_window: false,
  global_close_shortcut: '',
  capture_text_shortcut: 'Cmd+Shift+C',
  autostart_enabled: false,

  compression_max_kb: 200,
  edge_detection_enabled: true,
  edge_reaction_time_ms: 50,
  note_edge_open_delay_enabled: false,
  note_edge_open_delay_ms: 1000,
  edge_modifier_keys: [],
  edge_excluded_apps: [],
  overlay_strength: 10,
  window_blur: 55,
  window_saturation: 85,
  window_brightness: -85,
  text_color: '#ffffff',
  accent_color: '#8b5cf6',
  internal_link_color: '#a78bfa',
  external_link_color: '#60a5fa',
  notes_folder: 'Notes',
  pinned_notes: [],
  save_to_daily_shortcut: 'Cmd+Enter',
  save_as_note_shortcut: 'Cmd+Shift+Enter',
  append_to_note_shortcut: 'Cmd+Option+Enter',
  reader_shortcut: 'Cmd+Shift+R',
  reader_shortcut_closes_window: false,
  reader_close_shortcut: '',
  reader_open_in_obsidian_shortcut: 'Cmd+Shift+O',
  reader_edge_enabled: true,
  reader_edge_open_delay_enabled: false,
  reader_edge_open_delay_ms: 1000,
  reader_edge_side: 'left',
  reader_edge_modifier_keys: [],
  reader_hide_frontmatter: true,
  reader_hide_dataview: true,
  reader_hide_obsidian_comments: true,
  reader_hide_inline_fields: true,
  reader_hide_html: true,
  reader_hide_callouts: true,
  note_filename_template: 'note-YYYY-MM-DD-HHmmss',
  note_template: '---\ncreated: <% tp.date.now("YYYY-MM-DD hh:mm") %>\nmodified: \ndaily: "[[<% tp.date.now("YYYY-MM-DD") %>]]"\ntags: inbox\ntype: inbox\n---',
};

export const settings = writable({
  ...defaultSettings
});

export const captureContent = writable('');

export const isVisible = writable(false);

export const isLoading = writable(false);

export const errorMessage = writable('');

export const successMessage = writable('');

export const cssVars = derived(settings, ($settings) => ({
  '--window-width': `${$settings.window_width}px`,
  '--window-height': `${$settings.window_height}px`,
  '--border-radius': `${$settings.border_radius}px`,
  '--background-color': $settings.overlay_color,
  '--font-family': $settings.font_family,
  '--font-size': `${$settings.font_size}px`,
}));

export function showError(message, duration = 5000) {
  errorMessage.set(message);
  setTimeout(() => errorMessage.set(''), duration);
}

export function showSuccess(message, duration = 3000) {
  successMessage.set(message);
  setTimeout(() => successMessage.set(''), duration);
}
