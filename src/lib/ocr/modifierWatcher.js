import { getModifierFlags, isOcrModifierActive } from "./ocrClient.js";

export function createModifierWatcher({
    invoke,
    getIsDragging,
    getModifierSetting,
    onActiveChange,
}) {
    let interval = null;

    const start = () => {
        if (interval) return;
        const check = async () => {
            if (!getIsDragging()) {
                stop();
                return;
            }
            try {
                const flags = await getModifierFlags({ invoke });
                onActiveChange(isOcrModifierActive(flags, getModifierSetting()));
            } catch {
                onActiveChange(false);
            }
        };
        check();
        interval = setInterval(check, 80);
    };

    const stop = () => {
        if (interval) {
            clearInterval(interval);
            interval = null;
        }
        onActiveChange(false);
    };

    return { start, stop };
}
