use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::edge_detect::EdgeDetector;
use crate::settings::Settings;
use crate::shortcuts::ShortcutManager;
use crate::vault_index::VaultIndex;

pub(crate) struct AppState {
    pub(crate) settings: Arc<RwLock<Settings>>,
    pub(crate) edge_detector: Arc<EdgeDetector>,
    pub(crate) shortcut_manager: Arc<ShortcutManager>,
    pub(crate) reader_shortcut_manager: Arc<ShortcutManager>,
    pub(crate) capture_text_shortcut_manager: Arc<ShortcutManager>,
    pub(crate) save_as_note_shortcut_manager: Arc<ShortcutManager>,
    pub(crate) vault_index: Arc<RwLock<Option<VaultIndex>>>,
    pub(crate) window_geometry_debounce: Arc<AtomicU64>,
}
