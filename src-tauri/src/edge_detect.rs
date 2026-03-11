use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::settings::Settings;

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
        // Update enabled flag
        let mut e = self.enabled.write().await;
        *e = settings.edge_detection_enabled;
        drop(e);

        // Update settings
        let mut s = self.settings.write().await;
        *s = settings;

        log::info!(
            "Edge detection settings updated, enabled: {}",
            self.enabled.read().await
        );
    }

    /// Enable or disable edge detection
    pub async fn set_enabled(&self, enabled: bool) {
        let mut e = self.enabled.write().await;
        *e = enabled;
        log::info!("Edge detection enabled: {}", enabled);
    }

    /// Check if edge detection is enabled
    #[allow(dead_code)]
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
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

    /// Check if capture window is open
    #[allow(dead_code)]
    pub async fn is_capture_open(&self) -> bool {
        *self.is_capture_open.read().await
    }

    /// Check if reader window is open
    #[allow(dead_code)]
    pub async fn is_reader_open(&self) -> bool {
        *self.is_reader_open.read().await
    }

    /// Check if capture is in cooldown period after closing
    async fn is_capture_in_cooldown(&self) -> bool {
        if let Some(close_time) = *self.last_capture_close_time.read().await {
            close_time.elapsed() < Duration::from_millis(500)
        } else {
            false
        }
    }

    /// Check if reader is in cooldown period after closing
    async fn is_reader_in_cooldown(&self) -> bool {
        if let Some(close_time) = *self.last_reader_close_time.read().await {
            close_time.elapsed() < Duration::from_millis(500)
        } else {
            false
        }
    }

    /// Main polling loop - checks mouse position every 50ms
    async fn poll_loop(&self, app: AppHandle) {
        let poll_interval = Duration::from_millis(50);
        let mut trigger_start: Option<Instant> = None;
        let mut trigger_target: Option<&'static str> = None;
        let trigger_delay = Duration::from_millis(50);

        log::info!("Edge detection polling started");

        loop {
            tokio::time::sleep(poll_interval).await;

            // Get current mouse position and screen bounds
            let mouse_pos = get_mouse_position();
            let screen = get_primary_screen_bounds();

            let settings = self.settings.read().await;
            if !settings.edge_detection_enabled && !settings.reader_edge_enabled {
                trigger_start = None;
                trigger_target = None;
                continue;
            }
            let edge_zone = 5;
            let at_left_edge = mouse_pos.0 <= edge_zone;
            let at_right_edge = mouse_pos.0 >= screen.width - edge_zone;

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

            drop(settings); // Release lock early

            if let Some(target) = edge_target {
                if trigger_start.is_none() || trigger_target != Some(target) {
                    trigger_start = Some(Instant::now());
                    trigger_target = Some(target);
                } else if trigger_start.unwrap().elapsed() >= trigger_delay {
                    log::info!("Edge triggered! Emitting {}", target);
                    if target == "show_reader" {
                        *self.is_reader_open.write().await = true;
                    } else {
                        *self.is_capture_open.write().await = true;
                    }
                    let _ = app.emit(target, ());
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
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        let main_display = CGDisplay::main();
        let bounds = main_display.bounds();

        ScreenBounds {
            width: bounds.size.width as i32,
            height: bounds.size.height as i32,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        ScreenBounds {
            width: 1920,
            height: 1080,
        }
    }
}

/// Public function to get screen bounds (for use in other modules)
pub fn get_screen_bounds() -> (i32, i32) {
    let bounds = get_primary_screen_bounds();
    (bounds.width, bounds.height)
}
