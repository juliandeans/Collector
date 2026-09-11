<script>
    import { getCurrentWindow } from "@tauri-apps/api/window";

    let { onResizeStart = () => {} } = $props();

    // The window is borderless (decorations: false) and has
    // setMovableByWindowBackground enabled, so the native OS edge-resize
    // region isn't reliably reachable — dragging what looks like the edge
    // just moves the window instead. Driving Tauri's own resize-drag API
    // from this dedicated hit area resizes reliably regardless. preventDefault
    // + stopPropagation keep the same mousedown from also being picked up
    // as a window-background move.
    function startResize(event) {
        event.preventDefault();
        event.stopPropagation();
        onResizeStart();
        getCurrentWindow().startResizeDragging("SouthEast");
    }
</script>

<div
    class="resize-handle"
    onmousedown={startResize}
    role="presentation"
></div>

<style>
  .resize-handle {
    position: absolute;
    bottom: 4px;
    right: 4px;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
    opacity: 0.3;
    transition: opacity 0.2s;
  }

  .resize-handle::before {
    content: "";
    position: absolute;
    bottom: 0;
    right: 0;
    width: 12px;
    height: 12px;
    background:
      linear-gradient(
        135deg,
        transparent 40%,
        rgba(0, 0, 0, 0.2) 40%,
        rgba(0, 0, 0, 0.2) 45%,
        transparent 45%
      ),
      linear-gradient(
        135deg,
        transparent 50%,
        rgba(0, 0, 0, 0.2) 50%,
        rgba(0, 0, 0, 0.2) 55%,
        transparent 55%
      );
    border-radius: 0 0 14px 0;
  }

</style>
