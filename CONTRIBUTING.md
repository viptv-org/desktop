# Contributing to VIPTV desktop

Thanks for your interest. VIPTV is a multi-repository product; this repository owns the native Tauri v2 desktop shell for Linux, Windows and macOS.

## Workflow

1. Product behavior starts in [viptv-org/design](https://github.com/viptv-org/design). Read the pinned `DESIGN_REF` commit before changing app behavior.
2. Search this repository's GitHub Issues before opening a new one.
3. The UI is the pinned `tv` submodule of [viptv-org/tv-web](https://github.com/viptv-org/tv-web): advance the gitlink to a reviewed tv-web commit rather than editing anything under `tv/`. The local addon mode fat flavor is built with `VITE_VIPTV_LOCAL_MODE=1`.
4. Never commit credentials, tokens or provider URLs.
5. Validate before pushing: `npm run check` (cargo check) and `npm run test` (cargo test) inside `src-tauri`; full packaging additionally runs `npm run build`.

## License

Contributions are licensed under the GNU General Public License v2.0 only (see [LICENSE](LICENSE)). By contributing you agree your work is licensed under it.
