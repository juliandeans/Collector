use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::settings::Settings;

const WINDOW_CLOSE_COOLDOWN_MS: u64 = 500;
const EDGE_POLL_INTERVAL_MS: u64 = 50;
const EDGE_TRIGGER_ZONE_PX: i32 = 5;
#[cfg(not(target_os = "macos"))]
const FALLBACK_SCREEN_WIDTH_PX: i32 = 1920;
#[cfg(not(target_os = "macos"))]
const FALLBACK_SCREEN_HEIGHT_PX: i32 = 1080;

/// Edge detection state
pub struct EdgeDetector {
    /// Whether edge detection is enabled
    enabled: Arc<RwLock<bool>>,
    /// Whether the capture window is currently open
    is_capture_open: Arc<RwLock<bool>>,
    /// Whether the reader window is currently open
    is_reader_open: Arc<RwLock<bool>>,
    /// Current settings
    settings: Arc<RwLock<Settings>>,
    /// Cooldown timestamp for capture window
    last_capture_close_time: Arc<RwLock<Option<Instant>>>,
    /// Cooldown timestamp for reader window
    last_reader_close_time: Arc<RwLock<Option<Instant>>>,
}

/// Screen bounds
#[derive(Debug, Clone, Copy)]
struct ScreenBounds {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl EdgeDetector {
    /// Create a new EdgeDetector
    pub fn new(settings: Settings) -> Self {
        Self {
            enabled: Arc::new(RwLock::new(settings.edge_detection_enabled)),
            is_capture_open: Arc::new(RwLock::new(false)),
            is_reader_open: Arc::new(RwLock::new(false)),
            settings: Arc::new(RwLock::new(settings)),
            last_capture_close_time: Arc::new(RwLock::new(None)),
            last_reader_close_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the edge detection polling loop
    pub async fn start(self: Arc<Self>, app: AppHandle) {
        let detector = self.clone();

        tokio::spawn(async move {
            detector.poll_loop(app).await;
        });
    }

    /// Update settings
    pub async fn update_settings(&self, settings: Settings) {
        let mut e = self.enabled.write().await;
        *e = settings.edge_detection_enabled;
        drop(e);

        let mut s = self.settings.write().await;
        *s = settings;
    }

    /// Enable or disable edge detection
    pub async fn set_enabled(&self, enabled: bool) {
        let mut e = self.enabled.write().await;
        *e = enabled;
    }

    /// Set capture open state (called from main when capture visibility changes)
    pub async fn set_capture_open(&self, open: bool) {
        let mut w = self.is_capture_open.write().await;
        let was_open = *w;
        *w = open;

        if was_open && !open {
            *self.last_capture_close_time.write().await = Some(Instant::now());
        }
    }

    /// Set reader open state (called from main when reader visibility changes)
    pub async fn set_reader_open(&self, open: bool) {
        let mut w = self.is_reader_open.write().await;
        let was_open = *w;
        *w = open;

        if was_open && !open {
            *self.last_reader_close_time.write().await = Some(Instant::now());
        }
    }

    /// Check if capture is in cooldown period after closing
    async fn is_capture_in_cooldown(&self) -> bool {
        if let Some(close_time) = *self.last_capture_close_time.read().await {
            close_time.elapsed() < Duration::from_millis(WINDOW_CLOSE_COOLDOWN_MS)
        } else {
            false
        }
    }

    /// Check if reader is in cooldown period after closing
    async fn is_reader_in_cooldown(&self) -> bool {
        if let Some(close_time) = *self.last_reader_close_time.read().await {
            close_time.elapsed() < Duration::from_millis(WINDOW_CLOSE_COOLDOWN_MS)
        } else {
            false
        }
    }

    /// Main polling loop - checks mouse position on a short interval.
    async fn poll_loop(&self, app: AppHandle) {
        let poll_interval = Duration::from_millis(EDGE_POLL_INTERVAL_MS);
        let mut trigger_start: Option<Instant> = None;
        let mut trigger_target: Option<&'static str> = None;

        log::info!("Edge detection polling started");

        loop {
            tokio::time::sleep(poll_interval).await;

            let mouse_pos = get_mouse_position();
            let monitors = get_all_monitor_bounds();
            let current_monitor = get_monitor_for_cursor(mouse_pos, &monitors);
            let screen = current_monitor
                .map(|m| ScreenBounds {
                    width: m.width,
                    height: m.height,
                })
                .unwrap_or_else(get_primary_screen_bounds);

            let settings = self.settings.read().await.clone();
            if !settings.edge_detection_enabled && !settings.reader_edge_enabled {
                trigger_start = None;
                trigger_target = None;
                continue;
            }

            let trigger_delay = Duration::from_millis(settings.edge_reaction_time_ms);
            let edge_zone = EDGE_TRIGGER_ZONE_PX;
            let (at_left_edge, at_right_edge) = if let Some(m) = current_monitor {
                (
                    mouse_pos.0 <= m.x + edge_zone,
                    mouse_pos.0 >= m.x + m.width - edge_zone,
                )
            } else {
                (
                    mouse_pos.0 <= edge_zone,
                    mouse_pos.0 >= screen.width - edge_zone,
                )
            };

            let can_show_reader = settings.reader_edge_enabled
                && at_left_edge
                && !*self.is_reader_open.read().await
                && !self.is_reader_in_cooldown().await;
            let can_show_capture = settings.edge_detection_enabled
                && at_right_edge
                && !*self.is_capture_open.read().await
                && !self.is_capture_in_cooldown().await;

            let edge_target = if can_show_reader {
                Some("show_reader")
            } else if can_show_capture {
                Some("show_capture")
            } else {
                None
            };

            if let Some(target) = edge_target {
                if trigger_start.is_none() || trigger_target != Some(target) {
                    trigger_start = Some(Instant::now());
                    trigger_target = Some(target);
                } else if trigger_start.unwrap().elapsed() >= trigger_delay {
                    if !modifiers_match(&settings.edge_modifier_keys) {
                        trigger_start = None;
                        trigger_target = None;
                        continue;
                    }

                    if is_frontmost_app_excluded(&settings.edge_excluded_apps) {
                        trigger_start = None;
                        trigger_target = None;
                        continue;
                    }

                    if let Err(error) = app.emit(target, ()) {
                        log::warn!("Failed to emit edge event {}: {}", target, error);
                    }
                    trigger_start = None;
                    trigger_target = None;
                }
            } else {
                trigger_start = None;
                trigger_target = None;
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn modifiers_match(required: &[String]) -> bool {
    use core_graphics::event::CGEventFlags;
    use core_graphics::event_source::CGEventSourceStateID;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceFlagsState(state_id: CGEventSourceStateID) -> CGEventFlags;
    }

    if required.is_empty() {
        return true;
    }

    let flags = unsafe { CGEventSourceFlagsState(CGEventSourceStateID::HIDSystemState) };

    required
        .iter()
        .all(|modifier| match modifier.trim().to_lowercase().as_str() {
            "cmd" | "command" | "meta" => flags.contains(CGEventFlags::CGEventFlagCommand),
            "option" | "alt" => flags.contains(CGEventFlags::CGEventFlagAlternate),
            "shift" => flags.contains(CGEventFlags::CGEventFlagShift),
            "ctrl" | "control" => flags.contains(CGEventFlags::CGEventFlagControl),
            _ => true,
        })
}

#[cfg(not(target_os = "macos"))]
fn modifiers_match(_required: &[String]) -> bool {
    true
}

fn is_frontmost_app_excluded(excluded_apps: &[String]) -> bool {
    if excluded_apps.is_empty() {
        return false;
    }

    let frontmost = match get_frontmost_app() {
        Some(app) => app,
        None => return false,
    };

    excluded_apps
        .iter()
        .any(|excluded| excluded.trim().eq_ignore_ascii_case(&frontmost))
}

fn get_frontmost_app() -> Option<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to get name of first process where frontmost is true"#)
        .output()
        .ok()?;

    let app_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if app_name.is_empty() {
        None
    } else {
        Some(app_name)
    }
}

/// Get current mouse position using macOS Core Graphics
fn get_mouse_position() -> (i32, i32) {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
            if let Ok(event) = CGEvent::new(source) {
                let location = event.location();
                return (location.x as i32, location.y as i32);
            }
        }
        (0, 0)
    }

    #[cfg(not(target_os = "macos"))]
    {
        (0, 0)
    }
}

/// Get primary screen bounds using macOS Core Graphics
fn get_primary_screen_bounds() -> ScreenBounds {
    let bounds = get_primary_monitor_bounds();
    ScreenBounds {
        width: bounds.width,
        height: bounds.height,
    }
}

fn get_primary_monitor_bounds() -> MonitorBounds {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        let main_display = CGDisplay::main();
        let bounds = main_display.bounds();

        MonitorBounds {
            x: bounds.origin.x as i32,
            y: bounds.origin.y as i32,
            width: bounds.size.width as i32,
            height: bounds.size.height as i32,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        MonitorBounds {
            x: 0,
            y: 0,
            width: FALLBACK_SCREEN_WIDTH_PX,
            height: FALLBACK_SCREEN_HEIGHT_PX,
        }
    }
}

pub fn get_all_monitor_bounds() -> Vec<MonitorBounds> {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        if let Ok(displays) = CGDisplay::active_displays() {
            let monitors: Vec<MonitorBounds> = displays
                .into_iter()
                .map(|id| {
                    let bounds = CGDisplay::new(id).bounds();
                    MonitorBounds {
                        x: bounds.origin.x as i32,
                        y: bounds.origin.y as i32,
                        width: bounds.size.width as i32,
                        height: bounds.size.height as i32,
                    }
                })
                .collect();

            if !monitors.is_empty() {
                return monitors;
            }
        }
    }

    vec![get_primary_monitor_bounds()]
}

pub fn get_monitor_for_cursor<'a>(
    cursor: (i32, i32),
    monitors: &'a [MonitorBounds],
) -> Option<&'a MonitorBounds> {
    monitors.iter().find(|monitor| {
        cursor.0 >= monitor.x
            && cursor.0 < monitor.x + monitor.width
            && cursor.1 >= monitor.y
            && cursor.1 < monitor.y + monitor.height
    })
}

/// Public function to get screen bounds (for use in other modules)
pub fn get_screen_bounds() -> MonitorBounds {
    let cursor = get_mouse_position();
    let monitors = get_all_monitor_bounds();
    get_monitor_for_cursor(cursor, &monitors)
        .copied()
        .unwrap_or_else(get_primary_monitor_bounds)
}
