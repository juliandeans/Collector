//! Handle the resize border before TaoWindow::sendEvent starts a background
//! window drag. Tao 0.35's drag_resize_window is unsupported on macOS.
//! Only the resize border/grip is consumed; window-background moving stays on.
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use block::ConcreteBlock;
use cocoa::base::{id, nil, BOOL, YES};
use cocoa::foundation::{NSPoint, NSRect, NSSize};
use objc::{class, msg_send, sel, sel_impl};
use tauri::{Manager, WindowEvent};

use crate::resize_geometry::{hit_test, resize, Direction, Frame};

#[derive(Clone, Copy)]
struct Drag {
    start: Frame,
    mouse: NSPoint,
    direction: Direction,
    min: NSSize,
    max: NSSize,
}

pub(crate) fn install(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let alive = Arc::new(AtomicBool::new(true));
    let local_token = Arc::new(AtomicUsize::new(0));
    let global_token = Arc::new(AtomicUsize::new(0));
    let app = window.app_handle().clone();
    let alive_destroy = alive.clone();
    let local_destroy = local_token.clone();
    let global_destroy = global_token.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            alive_destroy.store(false, Ordering::SeqCst);
            let local = local_destroy.swap(0, Ordering::SeqCst);
            let global = global_destroy.swap(0, Ordering::SeqCst);
            let _ = app.run_on_main_thread(move || unsafe {
                for token in [local, global] {
                    if token != 0 {
                        let _: () = msg_send![class!(NSEvent), removeMonitor: token as id];
                    }
                }
            });
        }
    });
    let window_main = window.clone();
    window.run_on_main_thread(move || unsafe {
        if !alive.load(Ordering::SeqCst) { return; }
        let ns_window = match window_main.ns_window() {
            Ok(ptr) => ptr as id,
            Err(error) => { log::warn!("Cannot install native resize handler: {error}"); return; }
        };
        let drag: Rc<RefCell<Option<Drag>>> = Rc::new(RefCell::new(None));
        let local_drag = drag.clone();
        let local = ConcreteBlock::new(move |event: id| -> id {
            if !alive.load(Ordering::SeqCst) { return event; }
            let kind: u64 = msg_send![event, type];
            if kind == 1 { // LeftMouseDown
                let target: id = msg_send![event, window];
                if target != ns_window { return event; }
                *local_drag.borrow_mut() = None;
                // Leave modifier gestures (including BTT Option+Shift) alone.
                let modifiers: u64 = msg_send![event, modifierFlags];
                if modifiers & (0xF << 17) != 0 { return event; }
                let frame: NSRect = msg_send![ns_window, frame];
                let point: NSPoint = msg_send![event, locationInWindow];
                let resizable: u64 = msg_send![ns_window, styleMask];
                if resizable & 8 == 0 { return event; } // NSWindowStyleMaskResizable
                let Some(direction) = hit_test((point.x, point.y), (frame.size.width, frame.size.height)) else { return event; };
                *local_drag.borrow_mut() = Some(Drag {
                    start: Frame { x: frame.origin.x, y: frame.origin.y, width: frame.size.width, height: frame.size.height },
                    mouse: msg_send![class!(NSEvent), mouseLocation],
                    direction,
                    min: msg_send![ns_window, minSize],
                    max: msg_send![ns_window, maxSize],
                });
                return nil;
            }
            if local_drag.borrow().is_none() { return event; }
            if kind == 6 || kind == 2 { // LeftMouseDragged / final LeftMouseUp position
                let visible: BOOL = msg_send![ns_window, isVisible];
                if visible != YES { *local_drag.borrow_mut() = None; return event; }
                let active = local_drag.borrow().as_ref().copied().unwrap();
                if kind == 2 { *local_drag.borrow_mut() = None; }
                let mouse: NSPoint = msg_send![class!(NSEvent), mouseLocation];
                let frame = resize(active.start, active.direction,
                    (mouse.x - active.mouse.x, mouse.y - active.mouse.y),
                    (active.min.width, active.min.height), (active.max.width, active.max.height));
                let rect = NSRect::new(NSPoint::new(frame.x, frame.y), NSSize::new(frame.width, frame.height));
                // One atomic frame update, in the normal event loop. No nested
                // AppKit live-resize loop, CSS effect toggles, or split size/position.
                let _: () = msg_send![ns_window, setFrame: rect display: YES];
                return nil;
            }
            event
        }).copy();
        let token: id = msg_send![class!(NSEvent), addLocalMonitorForEventsMatchingMask: ((1u64 << 1) | (1u64 << 2) | (1u64 << 6)) handler: &*local];
        local_token.store(token as usize, Ordering::SeqCst);
        // Cancel if the release is delivered to a different application.
        // Global monitors observe only; they never replace another app's event.
        let global = ConcreteBlock::new(move |_event: id| { *drag.borrow_mut() = None; }).copy();
        let token: id = msg_send![class!(NSEvent), addGlobalMonitorForEventsMatchingMask: (1u64 << 2) handler: &*global];
        global_token.store(token as usize, Ordering::SeqCst);
    })
}
