import { invoke } from "@tauri-apps/api/core";

export async function loadTabContent(path) {
  return invoke("read_note_file", { path });
}

export async function saveTabContent(path, content) {
  return invoke("write_note_file", {
    path,
    content,
  });
}

export async function getDailyNotePath(settings = {}) {
  void settings;
  return invoke("get_daily_note_path");
}
