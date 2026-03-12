import { writable, derived } from 'svelte/store';

export const defaultSettings = {
  vault_name: 'Vault',
  edge_side: 'right',
  window_width: 300,
  window_height: 100,
  reader_width: 380,
  reader_height: 700,
  border_radius: 8,
  background_color: '#FFFFFF',
  font_family: 'SF Pro',
  font_size: 14,
  daily_note_folder: 'Journal/',
  daily_note_format: 'YYYY-MM-DD',
  daily_note_path: '',
  image_folder: 'Images/Screenshots',
  image_filename: 'screenshot-YYYY-MM-DD-HHmmss',
  default_image_width: '600',
  entry_header: '#### HH:mm',
  global_shortcut: 'Cmd+Shift+N',
  global_shortcut_closes_window: false,
  
  compression_max_kb: 200,
  edge_detection_enabled: true,
  edge_reaction_time_ms: 50,
  edge_modifier_keys: [],
  edge_excluded_apps: [],
  window_transparency: 55,
  window_blur: 80,
  window_saturation: 200,
  window_brightness: 0,
  text_color: '#ffffff',
  accent_color: '#8b5cf6',
  internal_link_color: '#a78bfa',
  external_link_color: '#60a5fa',
  notes_folder: 'Notes',
  pinned_notes: [],
  save_to_daily_shortcut: 'Cmd+Enter',
  save_as_note_shortcut: 'Cmd+Shift+Enter',
  reader_shortcut: 'Cmd+Shift+R',
  reader_shortcut_closes_window: false,
  reader_edge_enabled: true,
  reader_hide_frontmatter: true,
  reader_hide_dataview: true,
  reader_hide_obsidian_comments: true,
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
  '--background-color': $settings.background_color,
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
