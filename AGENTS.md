# Agent Instructions

## Validation

- After making code changes, run `cargo fmt`.
- After Rust code changes, run `cargo build`.
- If `cargo build` is blocked because a running `edolview` process is holding the executable open (for example from a VS Code debug session), terminate the `edolview` process and retry the build.

## UI verification

- Use `egui_inspection` for UI verification. Build with `cargo build --features eframe/inspection`, then set `EGUI_INSPECTION=127.0.0.1:<test-port>` in the test process's environment only. The feature and inspection listener are opt-in.
- Run a separate debug app instance without activating its window, isolate its settings/config directory, and preserve the user's existing instances and settings. Connect only to that instance's inspection port. If loading fixtures by image socket, use the test instance's address from stderr.
- Query the AccessKit tree to find widgets and inject egui input through inspection `ApplyEvents` for clicks, hover, drag, scroll, and keyboard input. Account for logical coordinates and pixels-per-point, and reproduce relevant boundary states.
- Never move the OS mouse, send global keyboard input, or steal window focus for UI verification. Do not use native input automation or Computer Use tools unless explicitly requested.
- Request captures through inspection `GetScreenshot` and directly inspect them before claiming visual correctness. Fully occluded or minimized windows may not render screenshots; report the limitation instead of bringing the test window to the foreground or falling back to native capture.
- Keep temporary inspection clients and captures under `target/`. Remove temporary clients and close only the test instance after verification; screenshots may remain for review. Report what was checked and any limitations.

## Generated assets and packaging

- If you touch `icon.svg`, `icons\`, `build.rs`, or startup icon loading, run `cargo run -p xtask -- icons` before validating.
- Keep generated icon files such as `icons\icon.png` and `icons\app.ico` in sync with code and packaging references.
- Treat files under `packaging\` as generated outputs unless you have confirmed otherwise.
- After changing packaging generation logic, regenerate assets with `cargo run -p xtask -- icons` so committed packaging outputs stay in sync with the source definitions.

## Resources

- Keep shared color definitions in `src\res\colors.rs`.
- Keep all keyboard shortcut definitions in `src\res\keyboard_shortcuts.rs`.
- When a keyboard shortcut has a corresponding button, always display the shortcut next to that button.
- Keep icon metadata and icon-related constants in `src\res\icons.rs`.

## Release and versioning

- Treat `Cargo.toml` as the single source of truth for the app version.
- If updating `Cargo.toml`, include the `Cargo.lock` change in the release commit.
- Prepare the version and lockfile, validate, commit, then push `main` and the matching `v<x.y.z>` tag.
- Do not create or push a release tag whose version does not match `Cargo.toml`.

## Unsafe, OpenGL, and image-core changes

- Treat `src\ui\gl\`, `src\ui\image_viewer.rs`, `src\model\image.rs`, and `src\model\image_io.rs` as safety-sensitive code.
- Keep `unsafe` blocks minimal and document invariants when changing them.

## Performance and concurrency rules

- This app is real-time. Prioritize low-latency interaction and immediate UI feedback.
- Keep heavy or latency-sensitive work off the egui UI thread.
- Preserve existing background-thread, channel, and `request_repaint` patterns.
- Avoid unnecessary full-image copies.
- Reuse existing shared ownership and caching patterns such as `Arc`, `Mutex`, `OnceLock`, and shared asset storage.

## Format support changes

- If you add a new image format, update the loader, the supported extension list in `src\model\file_nav.rs`, and the README supported-format list together.

## Socket protocol compatibility

- If you change the socket protocol, consider compatibility with the Python package and VS Code extension mentioned in the README.
- Document any breaking protocol change explicitly.

## Error handling

- Prefer existing `color_eyre` error propagation patterns.
- For user-visible failures, surface them through the current toast and `eprintln!` flow instead of failing silently.
