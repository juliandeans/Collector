//! On-demand freshness for the in-memory vault index (no file watcher).
//!
//! Callers ask for an index that is "fresh enough"; rebuilds run on a blocking
//! thread and are serialized so concurrent requests share one vault walk.

use std::time::{Duration, Instant};

use tokio::sync::{Mutex, RwLock};

use crate::settings::Settings;
use crate::vault_index::VaultIndex;

/// Minimum index age before a usage-triggered refresh walks the vault again.
pub(crate) const AUTO_REFRESH_MAX_AGE: Duration = Duration::from_secs(3);

/// Whether the index must be rebuilt for a request made at `requested_at`.
///
/// `max_age = None` rebuilds only when the index is missing or belongs to another
/// vault path. `Some(age)` additionally rebuilds when the index's walk started more
/// than `age` before the request; `Some(Duration::ZERO)` therefore accepts only a
/// walk that started at or after the request.
pub(crate) fn needs_rebuild(
    existing: Option<&VaultIndex>,
    vault_path: &str,
    max_age: Option<Duration>,
    requested_at: Instant,
) -> bool {
    let Some(index) = existing else {
        return true;
    };
    if index.vault_path != vault_path {
        return true;
    }
    match max_age {
        None => false,
        Some(age) => requested_at.saturating_duration_since(index.built_at) > age,
    }
}

/// Makes sure `index` holds an index for the current vault path that satisfies `max_age`.
pub(crate) async fn ensure_vault_index<B>(
    index: &RwLock<Option<VaultIndex>>,
    rebuild_lock: &Mutex<()>,
    settings: &RwLock<Settings>,
    max_age: Option<Duration>,
    build: B,
) -> Result<(), String>
where
    B: FnOnce(&str) -> Result<VaultIndex, String> + Send + 'static,
{
    let requested_at = Instant::now();
    // Lock-free fast path only for "build if missing" callers (images, tags, plain list).
    // Refresh requests always queue behind an in-flight rebuild, so an "ifStale" call that
    // arrives during a forced walk returns that walk's newer result, not the older index.
    if max_age.is_none() {
        let vault_path = settings.read().await.vault_path.clone();
        if !needs_rebuild(
            index.read().await.as_ref(),
            &vault_path,
            max_age,
            requested_at,
        ) {
            return Ok(());
        }
    }

    let _rebuild_guard = rebuild_lock.lock().await;

    // Another caller may have rebuilt while we waited, or the vault path may have changed.
    let vault_path = settings.read().await.vault_path.clone();
    if !needs_rebuild(
        index.read().await.as_ref(),
        &vault_path,
        max_age,
        requested_at,
    ) {
        return Ok(());
    }

    log::info!(
        "Building vault index for: {}",
        crate::log_safety::redact_path_str(&vault_path)
    );
    let build_path = vault_path.clone();
    let new_index = tokio::task::spawn_blocking(move || build(&build_path))
        .await
        .map_err(|error| format!("Vault index build task failed: {}", error))??;

    if settings.read().await.vault_path != vault_path {
        log::info!("Discarding vault index: vault path changed during build");
        return Err("Vault path changed while indexing".to_string());
    }

    log::info!(
        "Vault index built: {} files in {} ms",
        new_index.file_count,
        new_index.built_at.elapsed().as_millis()
    );
    *index.write().await = Some(new_index);
    Ok(())
}

/// Refresh mode requested by the frontend for `list_vault_notes`.
#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum NoteListRefresh {
    /// Usage-triggered: rebuild if the index is older than `AUTO_REFRESH_MAX_AGE`.
    IfStale,
    /// Rebuild unless a walk started after this request (unknown wikilink, manual reindex).
    Force,
}

impl NoteListRefresh {
    pub(crate) fn max_age(self) -> Duration {
        match self {
            Self::IfStale => AUTO_REFRESH_MAX_AGE,
            Self::Force => Duration::ZERO,
        }
    }
}

/// Like `ensure_vault_index`, but a failed rebuild keeps a usable previous index
/// for the current vault path instead of failing the request.
pub(crate) async fn refresh_keeping_stale<B>(
    index: &RwLock<Option<VaultIndex>>,
    rebuild_lock: &Mutex<()>,
    settings: &RwLock<Settings>,
    max_age: Duration,
    build: B,
) -> Result<(), String>
where
    B: FnOnce(&str) -> Result<VaultIndex, String> + Send + 'static,
{
    let Err(error) = ensure_vault_index(index, rebuild_lock, settings, Some(max_age), build).await
    else {
        return Ok(());
    };

    let vault_path = settings.read().await.vault_path.clone();
    let has_usable_index = index
        .read()
        .await
        .as_ref()
        .is_some_and(|existing| existing.vault_path == vault_path);
    if has_usable_index {
        log::warn!(
            "Vault index refresh failed, keeping previous index: {}",
            error
        );
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("collector-refresh-{label}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("First.md"), "# First").unwrap();
        dir
    }

    fn vault_path_string(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn settings_for(path: &Path) -> RwLock<Settings> {
        RwLock::new(Settings {
            vault_path: vault_path_string(path),
            ..Settings::default()
        })
    }

    async fn note_names(index: &RwLock<Option<VaultIndex>>) -> Vec<String> {
        index
            .read()
            .await
            .as_ref()
            .expect("index present")
            .all_notes()
            .into_iter()
            .map(|note| note.name)
            .collect()
    }

    fn backdated(mut index: VaultIndex, by: Duration) -> VaultIndex {
        index.built_at = Instant::now().checked_sub(by).unwrap();
        index
    }

    #[test]
    fn needs_rebuild_rules() {
        let vault = temp_vault("rules");
        let path = vault_path_string(&vault);
        let fresh = VaultIndex::build(&path).unwrap();
        let now = Instant::now();

        assert!(needs_rebuild(None, &path, None, now), "missing index");
        assert!(
            needs_rebuild(Some(&fresh), "/other/vault", None, now),
            "other vault"
        );
        assert!(!needs_rebuild(Some(&fresh), &path, None, now), "no max age");
        assert!(
            !needs_rebuild(Some(&fresh), &path, Some(AUTO_REFRESH_MAX_AGE), now),
            "fresh within 3 s"
        );

        let old = backdated(VaultIndex::build(&path).unwrap(), Duration::from_secs(10));
        assert!(
            needs_rebuild(
                Some(&old),
                &path,
                Some(AUTO_REFRESH_MAX_AGE),
                Instant::now()
            ),
            "older than 3 s"
        );

        let started_after_request = VaultIndex::build(&path).unwrap();
        let requested_before = started_after_request
            .built_at
            .checked_sub(Duration::from_millis(1))
            .unwrap();
        assert!(
            !needs_rebuild(
                Some(&started_after_request),
                &path,
                Some(Duration::ZERO),
                requested_before
            ),
            "force is satisfied by a walk that started after the request"
        );
    }

    #[tokio::test]
    async fn builds_when_missing() {
        let vault = temp_vault("missing");
        let index = RwLock::new(None);
        let lock = Mutex::new(());
        let settings = settings_for(&vault);

        ensure_vault_index(&index, &lock, &settings, None, VaultIndex::build)
            .await
            .unwrap();

        assert_eq!(note_names(&index).await, vec!["First".to_string()]);
    }

    #[tokio::test]
    async fn reuses_fresh_index_within_max_age() {
        let vault = temp_vault("fresh");
        let path = vault_path_string(&vault);
        let index = RwLock::new(Some(VaultIndex::build(&path).unwrap()));
        let lock = Mutex::new(());
        let settings = settings_for(&vault);
        fs::write(vault.join("Second.md"), "# Second").unwrap();
        let builds = Arc::new(AtomicUsize::new(0));
        let counter = builds.clone();

        ensure_vault_index(
            &index,
            &lock,
            &settings,
            Some(AUTO_REFRESH_MAX_AGE),
            move |path: &str| {
                counter.fetch_add(1, Ordering::SeqCst);
                VaultIndex::build(path)
            },
        )
        .await
        .unwrap();

        assert_eq!(builds.load(Ordering::SeqCst), 0);
        assert_eq!(note_names(&index).await, vec!["First".to_string()]);
    }

    #[tokio::test]
    async fn rebuilds_stale_index_and_sees_new_note() {
        let vault = temp_vault("stale");
        let path = vault_path_string(&vault);
        let old = backdated(VaultIndex::build(&path).unwrap(), Duration::from_secs(10));
        let index = RwLock::new(Some(old));
        let lock = Mutex::new(());
        let settings = settings_for(&vault);
        fs::write(vault.join("Second.md"), "# Second").unwrap();

        ensure_vault_index(
            &index,
            &lock,
            &settings,
            Some(AUTO_REFRESH_MAX_AGE),
            VaultIndex::build,
        )
        .await
        .unwrap();

        assert_eq!(
            note_names(&index).await,
            vec!["First".to_string(), "Second".to_string()]
        );
    }

    #[tokio::test]
    async fn force_rebuilds_even_a_fresh_index() {
        let vault = temp_vault("force");
        let path = vault_path_string(&vault);
        let index = RwLock::new(Some(VaultIndex::build(&path).unwrap()));
        let lock = Mutex::new(());
        let settings = settings_for(&vault);
        fs::write(vault.join("Second.md"), "# Second").unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;

        ensure_vault_index(
            &index,
            &lock,
            &settings,
            Some(Duration::ZERO),
            VaultIndex::build,
        )
        .await
        .unwrap();

        assert_eq!(
            note_names(&index).await,
            vec!["First".to_string(), "Second".to_string()]
        );
    }

    #[tokio::test]
    async fn concurrent_forced_refreshes_share_one_walk() {
        let vault = temp_vault("coalesce");
        let index = Arc::new(RwLock::new(None));
        let lock = Arc::new(Mutex::new(()));
        let settings = Arc::new(settings_for(&vault));
        let builds = Arc::new(AtomicUsize::new(0));

        // Hold the rebuild lock so both requests are registered before any walk starts.
        let held = lock.lock().await;
        let mut tasks = Vec::new();
        for _ in 0..2 {
            let (index, lock, settings, builds) = (
                index.clone(),
                lock.clone(),
                settings.clone(),
                builds.clone(),
            );
            tasks.push(tokio::spawn(async move {
                ensure_vault_index(
                    &index,
                    &lock,
                    &settings,
                    Some(Duration::ZERO),
                    move |path: &str| {
                        builds.fetch_add(1, Ordering::SeqCst);
                        VaultIndex::build(path)
                    },
                )
                .await
            }));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        drop(held);

        for task in tasks {
            task.await.unwrap().unwrap();
        }
        assert_eq!(builds.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn if_stale_request_waits_for_in_flight_rebuild() {
        let vault = temp_vault("wait");
        let path = vault_path_string(&vault);
        let index = Arc::new(RwLock::new(Some(VaultIndex::build(&path).unwrap())));
        let lock = Arc::new(Mutex::new(()));
        let settings = Arc::new(settings_for(&vault));

        // Simulate a forced rebuild in progress by holding the rebuild lock.
        let held = lock.lock().await;
        let task = {
            let (index, lock, settings) = (index.clone(), lock.clone(), settings.clone());
            tokio::spawn(async move {
                ensure_vault_index(
                    &index,
                    &lock,
                    &settings,
                    Some(AUTO_REFRESH_MAX_AGE),
                    VaultIndex::build,
                )
                .await
            })
        };
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !task.is_finished(),
            "ifStale must not return the old index while a rebuild is running"
        );

        drop(held);
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn plain_lookup_does_not_wait_for_in_flight_rebuild() {
        let vault = temp_vault("no-wait");
        let path = vault_path_string(&vault);
        let index = RwLock::new(Some(VaultIndex::build(&path).unwrap()));
        let lock = Mutex::new(());
        let settings = settings_for(&vault);

        let _held = lock.lock().await;
        tokio::time::timeout(
            Duration::from_millis(500),
            ensure_vault_index(&index, &lock, &settings, None, VaultIndex::build),
        )
        .await
        .expect("max_age None must use the lock-free fast path")
        .unwrap();
    }

    #[tokio::test]
    async fn discards_result_when_vault_path_changes_during_build() {
        let vault = temp_vault("path-change");
        let index = RwLock::new(None);
        let lock = Mutex::new(());
        let settings = Arc::new(settings_for(&vault));
        let settings_in_build = settings.clone();

        let result = ensure_vault_index(&index, &lock, &settings, None, move |path: &str| {
            let built = VaultIndex::build(path);
            settings_in_build.blocking_write().vault_path = "/somewhere/else".to_string();
            built
        })
        .await;

        assert_eq!(result.unwrap_err(), "Vault path changed while indexing");
        assert!(index.read().await.is_none());
    }

    #[tokio::test]
    async fn keeps_previous_index_when_refresh_fails() {
        let vault = temp_vault("keep-stale");
        let path = vault_path_string(&vault);
        let old = backdated(VaultIndex::build(&path).unwrap(), Duration::from_secs(10));
        let index = RwLock::new(Some(old));
        let lock = Mutex::new(());
        let settings = settings_for(&vault);

        refresh_keeping_stale(&index, &lock, &settings, AUTO_REFRESH_MAX_AGE, |_: &str| {
            Err("boom".to_string())
        })
        .await
        .unwrap();

        assert_eq!(note_names(&index).await, vec!["First".to_string()]);
    }

    #[tokio::test]
    async fn refresh_failure_without_usable_index_is_an_error() {
        let vault = temp_vault("no-index");
        let index = RwLock::new(None);
        let lock = Mutex::new(());
        let settings = settings_for(&vault);

        let result =
            refresh_keeping_stale(&index, &lock, &settings, AUTO_REFRESH_MAX_AGE, |_: &str| {
                Err("boom".to_string())
            })
            .await;

        assert_eq!(result.unwrap_err(), "boom");
    }

    #[test]
    fn note_list_refresh_wire_format() {
        let if_stale: NoteListRefresh = serde_json::from_str("\"ifStale\"").unwrap();
        let force: NoteListRefresh = serde_json::from_str("\"force\"").unwrap();
        assert_eq!(if_stale, NoteListRefresh::IfStale);
        assert_eq!(force, NoteListRefresh::Force);
        assert_eq!(if_stale.max_age(), AUTO_REFRESH_MAX_AGE);
        assert_eq!(force.max_age(), Duration::ZERO);
        assert!(serde_json::from_str::<NoteListRefresh>("\"always\"").is_err());
    }
}
