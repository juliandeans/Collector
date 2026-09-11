import "./global.css";
import { mount } from "svelte";
import Reader from "./Reader.svelte";

const app = mount(Reader, {
  target: document.getElementById("app"),
});

if (import.meta.env.DEV && import.meta.env.VITE_RESIZE_DIAGNOSTICS === "1") {
  import("./lib/resizeDiagnostics.js").then(({ mountResizeDiagnosticPanel }) => mountResizeDiagnosticPanel());
}

export default app;
