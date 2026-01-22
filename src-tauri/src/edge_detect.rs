use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};
use tokio::sync::RwLock;

use crate::settings::Settings;

/// Edge detection state
pub struct EdgeDetector {
    /// Whether edge detection is enabled
    enabled: Arc<RwLock<bool>>,
    /// Whether the capture window is currently open
    is_window_open: Arc<RwLock<bool>>,
    /// Current settings
    settings: Arc<RwLock<Settings>>,
    /// Cooldown timestamp
    last_close_time: Arc<RwLock<Option<Instant>>>,
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
            is_window_open: Arc::new(RwLock::new(false)),
            settings: Arc::new(RwLock::new(settings)),
            last_close_time: Arc::new(RwLock::new(None)),
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

        log::info!("Edge detection settings updated, enabled: {}", self.enabled.read().await);
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

    /// Set window open state (called from main when window visibility changes)
    pub async fn set_window_open(&self, open: bool) {
        let mut w = self.is_window_open.write().await;
        let was_open = *w;
        *w = open;

        // If window was just closed, set cooldown
        if was_open && !open {
            *self.last_close_time.write().await = Some(Instant::now());
        }
    }

    /// Check if window is open
    #[allow(dead_code)]
    pub async fn is_window_open(&self) -> bool {
        *self.is_window_open.read().await
    }

    /// Check if we're in cooldown period after closing
    async fn is_in_cooldown(&self) -> bool {
        if let Some(close_time) = *self.last_close_time.read().await {
            // 500ms cooldown after window close
            close_time.elapsed() < Duration::from_millis(500)
        } else {
            false
        }
    }

    /// Main polling loop - checks mouse position every 50ms
    async fn poll_loop(&self, app: AppHandle) {
        let poll_interval = Duration::from_millis(50);
        let mut trigger_start: Option<Instant> = None;
        let trigger_delay = Duration::from_millis(50);

        log::info!("Edge detection polling started");

        loop {
            tokio::time::sleep(poll_interval).await;

            // Check if enabled
            if !*self.enabled.read().await {
                trigger_start = None;
                continue;
            }

            // Check if in cooldown
            if self.is_in_cooldown().await {
                trigger_start = None;
                continue;
            }

            // Check if window is already open
            if *self.is_window_open.read().await {
                trigger_start = None;
                continue;
            }

            // Get current mouse position and screen bounds
            let mouse_pos = get_mouse_position();
            let screen = get_primary_screen_bounds();

            let settings = self.settings.read().await;
            let edge_zone = 5; // 5px trigger zone

            // Check if mouse is at the configured edge
            let at_edge = match settings.edge_side.as_str() {
                "right" => mouse_pos.0 >= screen.width - edge_zone,
                "left" => mouse_pos.0 <= edge_zone,
                _ => false,
            };

            drop(settings); // Release lock early

            if at_edge {
                // Mouse is at edge
                if trigger_start.is_none() {
                    // Start timing
                    trigger_start = Some(Instant::now());
                } else if trigger_start.unwrap().elapsed() >= trigger_delay {
                    // Delay passed, trigger window
                    log::info!("Edge triggered! Opening capture window");

                    // Mark window as open FIRST to prevent re-triggering
                    *self.is_window_open.write().await = true;

                    // Get settings for positioning
                    let settings = self.settings.read().await;
                    let width = settings.window_width as f64;
                    let height = settings.window_height as f64;
                    let edge_side = settings.edge_side.clone();
                    drop(settings);

                    let x = match edge_side.as_str() {
                        "left" => 0.0,
                        _ => screen.width as f64 - width,
                    };
                    let y = (screen.height as f64 - height) / 2.0;

                    // Position and show window
                    if let Some(window) = app.get_webview_window("capture") {
                        let _ = window
                            .set_size(tauri::LogicalSize { width, height });
                        let _ =
                            window.set_position(tauri::LogicalPosition {
                                x,
                                y,
                            });
                        let _ = window.show();
                        let _ = window.set_focus();
                    }

                    // Emit event for frontend to update UI state
                    let _ = app.emit("show_capture", ());

                    trigger_start = None;
                }
            } else {
                // Mouse not at edge, reset trigger
                trigger_start = None;
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
