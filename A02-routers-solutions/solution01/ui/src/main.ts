import { invoke } from "@tauri-apps/api/core";
import { createPieApp } from "./app";
import "./styles.css";

const root = document.querySelector<HTMLElement>("#app");

if (root) {
  createPieApp(root, invoke);
}
