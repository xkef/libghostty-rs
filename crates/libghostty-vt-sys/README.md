# libghostty-vt-sys

Raw FFI bindings for libghostty-vt.

- Fetches and builds `libghostty-vt.so`/`.dylib` from ghostty sources via Zig.
- Exposes checked-in generated bindings in `src/bindings.rs`.
- Enable the `link-static` feature to link the vendored `libghostty-vt.a`
  archive instead of the shared library.
- Set `GHOSTTY_SOURCE_DIR` to force the build to use a local Ghostty checkout.
- If the `pkg-config` feature is enabled, the build will use an installed
  `libghostty-vt` found through `pkg-config` only when `GHOSTTY_SOURCE_DIR` is
  unset. With `link-static`, it probes Ghostty's `libghostty-vt-static`
  pkg-config module instead.
- libghostty-vt is pre-1.0, so these bindings do not guarantee compatibility
  with arbitrary installed C API revisions.
