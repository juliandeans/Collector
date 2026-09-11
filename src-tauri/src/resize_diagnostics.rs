//! Temporary, opt-in macOS diagnostics. Never compiled into release builds.
//! Observe events without consuming them or changing any window flags.
use std::collections::{HashSet, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use block::ConcreteBlock;
use cocoa::base::{id, nil, BOOL, YES};
use cocoa::foundation::{NSPoint, NSRect, NSString};
use objc::{class, msg_send, sel, sel_impl};
use serde_json::{json, Value};

#[derive(Default)]
struct Trace {
    installed: HashSet<String>,
    active: HashSet<String>,
    entries: VecDeque<Value>,
}

fn trace() -> &'static Mutex<Trace> {
    static TRACE: OnceLock<Mutex<Trace>> = OnceLock::new();
    TRACE.get_or_init(|| Mutex::new(Trace::default()))
}

fn record(label: &str, kind: &str, details: Value) {
    let mut state = trace().lock().unwrap();
    if !state.active.contains(label) {
        return;
    }
    if state.entries.len() == 4096 {
        state.entries.pop_front();
    }
    state.entries.push_back(json!({
        "timeMs": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
        "label": label, "type": kind, "details": details,
    }));
}

// Only called from AppKit callbacks or run_on_main_thread.
unsafe fn snapshot(window: id) -> Value {
    let frame: NSRect = msg_send![window, frame];
    let live: BOOL = msg_send![window, inLiveResize];
    let movable: BOOL = msg_send![window, isMovableByWindowBackground];
    let mask: u64 = msg_send![window, styleMask];
    let mouse: NSPoint = msg_send![window, mouseLocationOutsideOfEventStream];
    let content: id = msg_send![window, contentView];
    let parent: id = msg_send![content, superview];
    let point: NSPoint = if parent.is_null() {
        mouse
    } else {
        msg_send![parent, convertPoint: mouse fromView: nil]
    };
    let hit: id = msg_send![content, hitTest: point];
    let hit_class = if hit.is_null() {
        "nil"
    } else {
        (*hit).class().name()
    };
    json!({
        "frame": [frame.origin.x, frame.origin.y, frame.size.width, frame.size.height],
        "mouseInWindowBottomLeft": [mouse.x, mouse.y],
        "inLiveResize": live == YES,
        "movableByWindowBackground": movable == YES,
        "styleMask": mask,
        "hitView": hit_class,
    })
}

unsafe fn install(window: id, label: &str) {
    {
        let mut state = trace().lock().unwrap();
        state.active.insert(label.to_string());
        if !state.installed.insert(label.to_string()) {
            return;
        }
    }
    // AppKit copies/owns these blocks. At most one observer set per persistent
    // Capture/Reader window lives until process exit. Inactive sets do no work.
    let center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
    for kind in [
        "NSWindowWillStartLiveResizeNotification",
        "NSWindowDidEndLiveResizeNotification",
        "NSWindowDidResizeNotification",
        "NSWindowWillMoveNotification",
        "NSWindowDidMoveNotification",
    ] {
        let label = label.to_string();
        let callback = ConcreteBlock::new(move |_notification: id| {
            if trace().lock().unwrap().active.contains(&label) {
                record(&label, kind, snapshot(window));
            }
        })
        .copy();
        let name = NSString::alloc(nil).init_str(kind);
        let _: id = msg_send![center, addObserverForName: name object: window queue: nil usingBlock: &*callback];
        let _: () = msg_send![name, release];
    }
    let label = label.to_string();
    let callback = ConcreteBlock::new(move |event: id| -> id {
        let target: id = msg_send![event, window];
        if target == window && trace().lock().unwrap().active.contains(&label) {
            let kind: u64 = msg_send![event, type];
            record(
                &label,
                if kind == 1 {
                    "native-mousedown"
                } else {
                    "native-mouseup"
                },
                snapshot(window),
            );
        }
        event // Never swallow/replace the event.
    })
    .copy();
    let mask: u64 = (1 << 1) | (1 << 2); // LeftMouseDown / LeftMouseUp only.
    let _: id =
        msg_send![class!(NSEvent), addLocalMonitorForEventsMatchingMask: mask handler: &*callback];
}

#[tauri::command]
pub(crate) async fn resize_diagnostics(
    window: tauri::WebviewWindow,
    action: String,
    frontend: Option<String>,
) -> Result<Value, String> {
    let label = window.label().to_string();
    if !matches!(label.as_str(), "capture" | "reader") {
        return Err("Resize diagnostics are for Capture and Reader only".into());
    }
    match action.as_str() {
        "start" | "snapshot" => {
            let (send, recv) = tokio::sync::oneshot::channel();
            let window_main = window.clone();
            window
                .run_on_main_thread(move || {
                    let result =
                        window_main
                            .ns_window()
                            .map_err(|e| e.to_string())
                            .map(|ptr| unsafe {
                                if action == "start" {
                                    install(ptr as id, &label);
                                }
                                let details = snapshot(ptr as id);
                                record(&label, "snapshot", details.clone());
                                details
                            });
                    let _ = send.send(result);
                })
                .map_err(|e| e.to_string())?;
            recv.await.map_err(|e| e.to_string())?
        }
        "stop" => {
            trace().lock().unwrap().active.remove(&label);
            Ok(json!(null))
        }
        "save" => {
            let frontend = frontend.ok_or("Missing frontend trace")?;
            if frontend.len() > 4 * 1024 * 1024 {
                return Err("Trace too large".into());
            }
            let frontend: Value = serde_json::from_str(&frontend).map_err(|e| e.to_string())?;
            let native: Vec<Value> = trace()
                .lock()
                .unwrap()
                .entries
                .iter()
                .filter(|entry| entry["label"] == label)
                .cloned()
                .collect();
            let report = json!({ "frontend": frontend, "native": native });
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos();
            let path = std::env::temp_dir().join(format!("collector-resize-{label}-{stamp}.json"));
            std::fs::write(
                &path,
                serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            log::info!("Resize diagnostic report: {}", path.display());
            Ok(json!({"path": path, "nativeEvents": native.len()}))
        }
        _ => Err("Unknown diagnostic action".into()),
    }
}
