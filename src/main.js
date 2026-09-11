import "./global.css";
import { mount } from "svelte";
import App from './App.svelte';

const app = mount(App, {
  target: document.getElementById('app'),
});

if (import.meta.env.DEV && import.meta.env.VITE_RESIZE_DIAGNOSTICS === "1") {
  import("./lib/resizeDiagnostics.js").then(({ mountResizeDiagnosticPanel }) => mountResizeDiagnosticPanel());
}

export default app;
