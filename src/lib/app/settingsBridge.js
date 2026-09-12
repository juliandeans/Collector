import { defaultSettings } from "../stores.js";

export function computeBrightnessFilter(brightness = 0) {
    if (brightness === 0) return " brightness(1)";

    if (brightness > 0) {
        const brightnessValue = 1 + (brightness / 100) * 0.6;
        const contrastValue = 1 - (brightness / 100) * 0.25;
        return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
    }

    const brightnessValue = 1 + (brightness / 100) * 0.7;
    const contrastValue = 1 + (-brightness / 100) * 0.3;
    return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
}

export function applyColorSettings(settings = {}) {
    const root = document.documentElement;
    root.style.setProperty(
        "--text-secondary",
        `color-mix(in srgb, ${settings.text_color ?? "#ffffff"} 50%, transparent)`,
    );
    root.style.setProperty(
        "--accent-color",
        settings.accent_color ?? "#8b5cf6",
    );
    root.style.setProperty(
        "--internal-link-color",
        settings.internal_link_color ?? "#a78bfa",
    );
    root.style.setProperty(
        "--external-link-color",
        settings.external_link_color ?? "#60a5fa",
    );
}

export function buildInitialAppSettings(settings) {
    return {
        overlay_color: settings.overlay_color,
        font_family: settings.font_family,
        font_size: settings.font_size,
        border_radius: settings.border_radius,
        edge_side: settings.edge_side ?? defaultSettings.edge_side,
        round_all_corners:
            settings.round_all_corners ?? defaultSettings.round_all_corners,
        overlay_strength:
            settings.overlay_strength ?? defaultSettings.overlay_strength,
        window_blur: settings.window_blur ?? defaultSettings.window_blur,
        window_saturation:
            settings.window_saturation ?? defaultSettings.window_saturation,
        window_brightness:
            settings.window_brightness ?? defaultSettings.window_brightness,
        text_color: settings.text_color ?? defaultSettings.text_color,
        accent_color: settings.accent_color ?? defaultSettings.accent_color,
        internal_link_color:
            settings.internal_link_color ?? defaultSettings.internal_link_color,
        external_link_color:
            settings.external_link_color ?? defaultSettings.external_link_color,
        entry_header: settings.entry_header ?? defaultSettings.entry_header,
        capture_rules: Array.isArray(settings.capture_rules)
            ? settings.capture_rules
            : [],
        show_note_paths:
            settings.show_note_paths ?? defaultSettings.show_note_paths,
        autocomplete_results:
            settings.autocomplete_results ??
            defaultSettings.autocomplete_results,
        save_to_daily_shortcut: settings.save_to_daily_shortcut ?? "Cmd+Enter",
        save_as_note_shortcut:
            settings.save_as_note_shortcut ?? "Cmd+Shift+Enter",
        append_to_note_shortcut:
            settings.append_to_note_shortcut ?? "Cmd+Option+Enter",
        show_capture_action_bar: settings.show_capture_action_bar ?? true,
        show_capture_mode_button: settings.show_capture_mode_button ?? true,
        capture_mode_shortcut:
            settings.capture_mode_shortcut ??
            defaultSettings.capture_mode_shortcut,
        persist_capture_draft_shortcut:
            settings.persist_capture_draft_shortcut ??
            defaultSettings.persist_capture_draft_shortcut,
        clear_capture_shortcut:
            settings.clear_capture_shortcut ?? defaultSettings.clear_capture_shortcut,
        ocr_shortcut: settings.ocr_shortcut ?? defaultSettings.ocr_shortcut,
        ocr_drag_modifier:
            settings.ocr_drag_modifier ?? defaultSettings.ocr_drag_modifier,
    };
}

export async function loadInitialSettings({ invoke, setAppSettings }) {
    try {
        const settings = await invoke("load_settings");
        const appSettings = buildInitialAppSettings(settings);
        setAppSettings(appSettings);
        applyColorSettings(appSettings);
    } catch (e) {
        console.error("Failed to load initial settings:", e);
    }
}

export function applySettingsUpdate(current, incoming) {
    return {
        ...current,
        overlay_color: incoming.overlay_color ?? current.overlay_color,
        font_family: incoming.font_family ?? current.font_family,
        font_size: incoming.font_size ?? current.font_size,
        border_radius: incoming.border_radius ?? current.border_radius,
        edge_side: incoming.edge_side ?? current.edge_side,
        round_all_corners:
            incoming.round_all_corners ?? current.round_all_corners,
        overlay_strength: incoming.overlay_strength ?? current.overlay_strength,
        window_blur: incoming.window_blur ?? current.window_blur,
        window_saturation:
            incoming.window_saturation ?? current.window_saturation,
        window_brightness:
            incoming.window_brightness ?? current.window_brightness,
        text_color: incoming.text_color ?? current.text_color,
        accent_color: incoming.accent_color ?? current.accent_color,
        internal_link_color:
            incoming.internal_link_color ?? current.internal_link_color,
        external_link_color:
            incoming.external_link_color ?? current.external_link_color,
        entry_header: incoming.entry_header ?? current.entry_header,
        capture_rules: Array.isArray(incoming.capture_rules)
            ? incoming.capture_rules
            : (current.capture_rules ?? []),
        show_note_paths: incoming.show_note_paths ?? current.show_note_paths,
        autocomplete_results:
            incoming.autocomplete_results ?? current.autocomplete_results,
        save_to_daily_shortcut:
            incoming.save_to_daily_shortcut ?? current.save_to_daily_shortcut,
        save_as_note_shortcut:
            incoming.save_as_note_shortcut ?? current.save_as_note_shortcut,
        append_to_note_shortcut:
            incoming.append_to_note_shortcut ?? current.append_to_note_shortcut,
        show_capture_action_bar:
            incoming.show_capture_action_bar ?? current.show_capture_action_bar,
        show_capture_mode_button:
            incoming.show_capture_mode_button ??
            current.show_capture_mode_button,
        capture_mode_shortcut:
            incoming.capture_mode_shortcut ?? current.capture_mode_shortcut,
        persist_capture_draft_shortcut:
            incoming.persist_capture_draft_shortcut ??
            current.persist_capture_draft_shortcut,
        clear_capture_shortcut:
            incoming.clear_capture_shortcut ?? current.clear_capture_shortcut,
        ocr_shortcut: incoming.ocr_shortcut ?? current.ocr_shortcut,
        ocr_drag_modifier:
            incoming.ocr_drag_modifier ?? current.ocr_drag_modifier,
    };
}
