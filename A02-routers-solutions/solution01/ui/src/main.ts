import { invoke } from "@tauri-apps/api/core";
import { createRouterWorkbenchApp } from "./app";
import "./styles.css";

const root = document.querySelector<HTMLElement>("#app");

if (root) {
  createRouterWorkbenchApp(root, invoke);
}
