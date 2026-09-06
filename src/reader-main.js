import "./global.css";
import { mount } from "svelte";
import Reader from "./Reader.svelte";

const app = mount(Reader, {
  target: document.getElementById("app"),
});

export default app;
