# Extraction provenance

The initial standalone Planet Engine foundation is pinned to `Zetic/Project-Interlink` commit `4cd8d727b5d437396a0da0fe652b64685a3bc309`, the completed WG-3.75 PR #104 head.

The extracted repository intentionally owns only Planet Engine physics, diagnostics, browser/WASM transport, and worldgen documentation. Project Interlink gameplay, Regions/Features, machine simulation, routing, and industrial-runtime code remain outside this repository.

Pull-request validation is likewise isolated: Planet Engine browser regressions and the three worldgen Rust crates run independently from Project Interlink. Committed WASM byte parity is a separate path-filtered gate so browser/docs-only changes avoid the slower wasm-bindgen packaging step.
