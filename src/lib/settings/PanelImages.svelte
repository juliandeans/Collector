<script>
    import { open } from "@tauri-apps/plugin-dialog";
    import { normalizeComparablePath } from "./path-utils.js";

    export let settings;
    export let showStatus;

    function toRelativeVaultDirectoryPath(path = "") {
        const normalizedPath = normalizeComparablePath(path.trim());
        const normalizedVaultPath = normalizeComparablePath(
            settings.vault_path ?? "",
        );

        if (!normalizedPath || !normalizedVaultPath) {
            return "";
        }

        if (normalizedPath === normalizedVaultPath) {
            return ".";
        }

        if (normalizedPath.startsWith(`${normalizedVaultPath}/`)) {
            return normalizedPath.slice(normalizedVaultPath.length + 1);
        }

        return "";
    }

    function resolveVaultSettingPath(path = "") {
        const rawPath = path.trim();
        const normalizedVaultPath = normalizeComparablePath(
            settings.vault_path ?? "",
        );

        if (!rawPath) {
            return normalizedVaultPath || "";
        }

        if (
            rawPath.startsWith("/") ||
            /^[A-Za-z]:[\\/]/.test(rawPath)
        ) {
            return rawPath;
        }

        if (!normalizedVaultPath) {
            return rawPath;
        }

        if (rawPath === ".") {
            return normalizedVaultPath;
        }

        return `${normalizedVaultPath}/${normalizeComparablePath(rawPath)}`;
    }

    async function pickScreenshotPath() {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath:
                resolveVaultSettingPath(settings.screenshot_path) ||
                settings.vault_path ||
                undefined,
        });
        if (selected) {
            const relative = toRelativeVaultDirectoryPath(selected);
            if (!relative) {
                showStatus(
                    "Image folder must be inside the current vault",
                    "error",
                );
                return;
            }

            settings.screenshot_path = relative;
            settings = { ...settings };
        }
    }
</script>

<div class="settings-panel">
    <section>
        <h2>Storage</h2>
        <div class="field">
            <label for="screenshot_path">Image Folder</label>
            <div class="path-picker">
                <input
                    type="text"
                    id="screenshot_path"
                    bind:value={settings.screenshot_path}
                    placeholder="Grafiken/Screenshots"
                />
                <button class="secondary" on:click={pickScreenshotPath}
                    >Choose...</button
                >
            </div>
            <small
                >Relative path in the vault for saved images (folder will be
                created automatically)</small
            >
        </div>
        <div class="field">
            <label for="image_filename">Filename Template</label>
            <input
                type="text"
                id="image_filename"
                bind:value={settings.image_filename}
                placeholder="screenshot-YYYY-MM-DD-HHmmss"
            />
            <small>Supports: YYYY, MM, DD, HH, mm, ss</small>
        </div>
    </section>

    <section>
        <h2>Embed Defaults</h2>
        <div class="field">
            <label for="compression_max_kb">Max. Image Size (KB)</label>
            <input
                type="number"
                id="compression_max_kb"
                bind:value={settings.compression_max_kb}
                min="50"
                max="1000"
                step="50"
            />
            <small>Images will be compressed to this size</small>
        </div>
        <div class="field">
            <label for="default_image_width">Default Image Width</label>
            <input
                type="text"
                id="default_image_width"
                bind:value={settings.default_image_width}
                placeholder="600"
                inputmode="numeric"
            />
            <small
                >Optional width in pixels for new image links (leave empty for
                no width)</small
            >
        </div>
    </section>
</div>
