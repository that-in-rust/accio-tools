# accio-tools

## Ten-Line Core Note

1. This repo contains the Tool Router Evidence Console for the tool-routing take-home.
2. The app starts with a benchmark inquiry or a custom inquiry from the reviewer.
3. The user chooses one CPU router: Lexical BM25, Schema-aware BM25, or Hybrid RRF.
4. The selected CPU router ranks the catalog and returns a top-five shortlist.
5. A low-cost LLM judge reviews only that top-five evidence and selects one tool or abstains.
6. Benchmark queries compare the judge decision against the bundled gold label.
7. The bundled pack is a curated OSS/reference subset with 947 tools and 50 queries.
8. The desktop shell uses Tauri 2 with a Rust Cargo workspace backend.
9. The UI uses TypeScript, Vite, plain DOM rendering, Vitest, and Playwright layout checks.
10. The final route decision requires a validated OpenAI API key; the run button stays disabled without it.

## OpenAI Key Required

The desktop app can open without a key, but `Run Selected Route Decision` will not run until an OpenAI API key is entered and validated in the Judge Session panel.

## Build Instructions

```bash
cd A02-routers-solutions/solution01
cargo test --workspace
cargo build --workspace
npm --prefix ui test
npm --prefix ui run build
npm --prefix ui run test:responsive-layout-viewports
```

## Run The App

```bash
cd A02-routers-solutions/solution01
npm --prefix ui run tauri:dev
```

## Package The App

```bash
cd A02-routers-solutions/solution01
npm --prefix ui run tauri:build
```
