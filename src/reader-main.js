import "./global.css";
import Reader from "./Reader.svelte";

const app = new Reader({
  target: document.getElementById("app"),
});

export default app;
