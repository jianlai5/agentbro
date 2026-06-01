# AgentBro Windows Port Plan

Date: 2026-06-01

## Purpose

This document is the handoff plan for adding Windows support to AgentBro.
AgentBro currently targets macOS first. The Windows port should proceed in small,
verifiable steps: compile first, then restore core behavior, then add native
Windows polish and packaging.

## How To Continue In A New Window

1. Read `AGENTS.md` first.
2. Read this document next.
3. Run `git status --short --branch` and do not revert unrelated user changes.
4. Find the first row whose `Done` value is `No`.
5. Work only on that row or a tightly related blocker.
6. After finishing a row, update its `Done` value to `Yes` and add notes under
   the corresponding phase.
7. Before moving to the next phase, run the verification command listed for that
   phase when it is available on the current machine.

Status values:

- `No`: not done yet.
- `In progress`: work has started but is not verified.
- `Blocked`: cannot continue without a decision, missing dependency, or platform
  access.
- `Yes`: implemented and verified.

## Current Assumptions

- Keep macOS behavior working while adding Windows support.
- Do not introduce new dependencies unless the tradeoff is discussed first.
- Prefer platform modules over scattered platform checks when behavior differs.
- Windows MVP can temporarily degrade non-core features with clear errors or
  safe fallback values.
- Core MVP means the app can start, show the floating surface, install hooks,
  receive bridge events, and pass local checks.

## Phase Summary

| Phase | Goal | Done | Verification |
| --- | --- | --- | --- |
| 0 | Establish baseline and compile failure list | Yes | Captured environment blockers on 2026-06-01 |
| 1 | Make Rust compile on Windows | In progress | Source fallbacks added; verification blocked by missing Windows build tools |
| 2 | Make frontend and bridge build cross-platform | In progress | Cross-platform bridge script added; verification blocked by missing pnpm/build tools |
| 3 | Stabilize Windows window, tray, and display behavior | No | App opens on Windows without crash |
| 4 | Make hook install and bridge command paths Windows-safe | No | Hook event reaches local HookServer |
| 5 | Add or degrade terminal focus and jump behavior | No | Commands return predictable results |
| 6 | Verify agent detection, session parsing, and watchers | No | Supported agents detected and sessions update |
| 7 | Add Windows packaging and updater path | No | Windows bundle builds locally or in CI |
| 8 | Final verification and regression pass | No | Required checks pass on macOS and Windows |

## Phase 0: Baseline

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Capture current branch, dirty files, Node, pnpm, Rust, and Tauri versions | repo root | Yes | Branch `windows-support`; Node `v25.8.2`; default Rust toolchain `stable-x86_64-pc-windows-gnu`; MSVC toolchain installed but build tools missing; `pnpm` and `corepack` not on PATH. |
| Run Windows `cargo check` and save the first meaningful compile errors | `src-tauri/` | Yes | GNU fails before project code because `dlltool.exe` is missing. MSVC fails before project code because `link.exe` and Windows SDK import libs such as `kernel32.lib` are missing. |
| Run `pnpm build` on Windows and capture script failures | `package.json`, scripts | Yes | `pnpm` is not recognized in the current environment. |
| Identify runtime-only Windows risks that compile checks will not catch | `src-tauri/src/platform`, `src-tauri/src/terminal`, frontend window code | Yes | Transparent window, tray, DPI, click-through behavior, and terminal focus/jump need real Windows desktop testing after toolchain install. |

Phase 0 verification:

```bash
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
```

Environment blockers captured on 2026-06-01:

- `cargo check --manifest-path src-tauri/Cargo.toml` with the default GNU toolchain
  stops at `Error calling dlltool 'dlltool.exe': program not found`.
- `cargo +stable-x86_64-pc-windows-msvc check --manifest-path src-tauri/Cargo.toml`
  stops at `linker 'link.exe' not found`.
- Using `rust-lld` with the MSVC toolchain still stops before project code because
  Windows SDK import libraries such as `kernel32.lib`, `ntdll.lib`, `userenv.lib`,
  `ws2_32.lib`, and `dbghelp.lib` are not installed.
- `pnpm build` cannot run because `pnpm` is not on PATH.

To continue verification, install one complete Windows Rust build path:

- Recommended: Visual Studio Build Tools with the Visual C++ workload and Windows
  SDK, then run `cargo +stable-x86_64-pc-windows-msvc check --manifest-path
  src-tauri/Cargo.toml`.
- Alternative: install a complete MinGW toolchain that provides `dlltool.exe`,
  then run `cargo check --manifest-path src-tauri/Cargo.toml` with the GNU target.
- Install or enable pnpm, then run `pnpm build` and `pnpm build:bridge`.

## Phase 1: Rust Compile Gate

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Ensure macOS-only crates and imports are behind macOS cfg gates | `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/platform/*` | In progress | Existing macOS-only `objc2` dependencies are target-gated. Full verification is blocked by missing Windows build tools. |
| Remove unconditional Unix-only imports and APIs from Windows builds | `src-tauri/src/terminal/wave.rs`, `src-tauri/src/hooks/server.rs`, `src-tauri/src/main.rs` | In progress | HookServer and bridge now use Unix sockets only on Unix; Windows falls back to TCP. Wave RPC returns unsupported on non-Unix. |
| Add safe Windows fallbacks for macOS-only commands | `src-tauri/src/terminal/suppression.rs`, `src-tauri/src/terminal/jump.rs`, `src-tauri/src/platform/idle.rs` | No | It is acceptable for MVP to return `false` or an unsupported error. |
| Keep shared types and Tauri command signatures stable | `src-tauri/src/lib.rs`, `src/services/tauriApi.ts` | In progress | Existing commands preserved; Hook Doctor reports Unix socket as `skip` on non-Unix. |

Phase 1 verification:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

## Phase 2: Cross-Platform Build Scripts And Bridge Resource

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Replace Unix shell bridge build commands with a cross-platform script | `package.json`, `scripts/` | Yes | Added `scripts/build-bridge.mjs`; `build:bridge` and `build:bridge:release` now call Node. |
| Handle `agentbro-bridge.exe` on Windows and `agentbro-bridge` on Unix | `package.json`, `src-tauri/tauri.conf.json`, bridge deployment code | In progress | Hook manager now installs `.exe` on Windows and Tauri resources include both names. Needs real build verification. |
| Verify debug and release bridge builds | `src-tauri/src/bridge/main.rs`, `src-tauri/target/agentbro-bridge-resource` | Blocked | Blocked by missing pnpm and Windows Rust build tools. |
| Avoid committing generated output | `src-tauri/target`, `dist`, `node_modules` | No | Generated paths stay untracked. |

Phase 2 verification:

```bash
pnpm build
pnpm build:bridge
```

## Phase 3: Windows Window, Tray, And Display Behavior

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Verify transparent frameless floating window on Windows WebView2 | `src-tauri/tauri.conf.json`, `src/components/notch/*` | No | Check visual transparency and resize behavior. |
| Verify click-through and cursor event toggling | `src-tauri/src/lib.rs`, `src/components/notch/NotchPanel.tsx` | No | Existing `set_ignore_cursor_events` behavior may differ by OS. |
| Verify tray icon and tray menu behavior | `src-tauri/src/lib.rs`, `src-tauri/icons/icon.ico` | No | Do not modify brand assets unless explicitly required. |
| Verify display positioning with DPI scaling and multi-monitor layouts | `src-tauri/src/platform/display.rs`, `src/components/notch/petStageAnchor.ts` | No | Test 100%, 125%, 150%, and negative monitor coordinates. |
| Decide whether Windows uses the same top-notch layout or a Windows-specific HUD default | `src/stores/configStore.ts`, settings UI | No | Keep MVP simple unless UX breaks. |

Phase 3 verification:

```bash
pnpm tauri:dev
```

Manual checks:

- Window appears on launch.
- Tray opens the app.
- Floating surface can be shown, hidden, dragged, and resized by existing UI flows.
- No invisible window blocks normal desktop clicks.

## Phase 4: Hook Install, Bridge Deployment, And Command Quoting

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Make bridge deployment choose the correct executable name per OS | `src-tauri/src/agents/hook_manager.rs`, `src-tauri/src/agents/profiles.rs` | No | Windows command should point to `.exe`. |
| Replace Unix command prefixes such as `/usr/bin/env` where Windows hooks need different syntax | `src-tauri/src/agents/profiles.rs`, `src-tauri/src/agents/toml_hooks.rs` | No | Be careful with spaces in `%USERPROFILE%` paths. |
| Audit JSON, TOML, and YAML hook command escaping on Windows | `src-tauri/src/agents/*`, tests | No | Add tests for paths containing spaces. |
| Verify hook install and uninstall for at least Claude Code and Codex | `src-tauri/src/agents/claude_code.rs`, `src-tauri/src/agents/codex.rs` | No | Exact support depends on those CLIs' Windows config format. |
| Verify HookServer transport works on Windows | `src-tauri/src/hooks/server.rs`, `src-tauri/src/hook_endpoint.rs`, bridge code | No | Unix sockets may need TCP or named-pipe fallback. |

Phase 4 verification:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Manual checks:

- Install hook for a supported agent.
- Start that agent.
- Confirm AgentBro receives at least one event.
- Uninstall hook and confirm config is restored.

## Phase 5: Terminal Focus Suppression And Jump Behavior

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Make terminal focus suppression compile and return deterministic values on Windows | `src-tauri/src/terminal/suppression.rs` | No | MVP can disable smart suppression on Windows. |
| Decide Windows Terminal support scope | `src-tauri/src/terminal/jump.rs`, `src-tauri/src/terminal/registry.rs` | No | Windows Terminal, PowerShell, cmd, Git Bash, WSL, and WezTerm differ. |
| Implement or safely degrade terminal jump commands | `src-tauri/src/terminal/jump.rs` | No | Return clear unsupported results rather than failing unpredictably. |
| Add focused tests for Windows command/path parsing if implemented | `src-tauri/src/terminal/*` | No | Avoid tests that require a real desktop session unless marked manual. |

Phase 5 verification:

```bash
cargo test --manifest-path src-tauri/Cargo.toml terminal
```

Manual checks:

- `is_terminal_focused` does not crash.
- `jump_to_terminal` either works for supported terminals or reports unsupported.

## Phase 6: Agent Detection, Sessions, And Watchers

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Verify home-directory based config discovery on Windows | `src-tauri/src/agents/detection.rs`, `src-tauri/src/skills/agent_paths.rs` | No | `dirs::home_dir()` should be preferred over raw env assumptions. |
| Verify Claude and Codex session file paths | `src-tauri/src/hooks/conversation_parser.rs`, `src-tauri/src/hooks/file_watcher.rs` | No | Confirm actual Windows paths used by each CLI. |
| Verify file watcher behavior on Windows | `src-tauri/src/hooks/file_watcher.rs` | No | Watcher limits and path normalization may differ. |
| Verify skills and plugin scanning path behavior | `src-tauri/src/skills/*` | No | Path separators should not leak into matching logic. |
| Verify network monitor and local HTTP services | `src-tauri/src/network_monitor.rs`, `src-tauri/src/hooks/server.rs` | No | Firewall prompts may appear on Windows. |

Phase 6 verification:

```bash
pnpm test:run
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Manual checks:

- Detected agent list is reasonable.
- Existing sessions appear.
- New hook/session events update the UI.

## Phase 7: Packaging, Updater, And CI

| Task | Likely Files | Done | Notes |
| --- | --- | --- | --- |
| Choose Windows bundle targets | `src-tauri/tauri.conf.json`, docs/release docs | In progress | Added Windows packaging workflow job targeting both `nsis` and `msi`; keep both until real installer testing shows one should be dropped. |
| Verify WebView2 runtime strategy | `src-tauri/tauri.conf.json` | No | Decide whether to rely on installed runtime or bundle bootstrap behavior. |
| Ensure updater metadata can distinguish Windows assets | release workflow, updater config | No | Avoid breaking existing macOS update flow. |
| Add Windows CI check path | `.github/workflows/*` | In progress | Added `.github/workflows/windows-compile.yml` for Windows compile checks on PRs and a manual packaging job that uploads installer artifacts; pending first successful GitHub Actions run. |
| Document Windows install limitations | `docs/` | No | Mention MVP gaps if any remain. |

Phase 7 verification:

```bash
pnpm tauri:build
```

Manual checks:

- Installer launches.
- Installed app starts from Start Menu or install directory.
- Tray and floating surface still work after installation.

## Phase 8: Final Verification

| Task | Done | Notes |
| --- | --- | --- |
| Run frontend lint | No | `pnpm lint` |
| Run frontend tests | No | `pnpm test:run` |
| Run frontend build | No | `pnpm build` |
| Run Rust check | No | `cargo check --manifest-path src-tauri/Cargo.toml` |
| Run Rust tests where practical | No | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Run macOS smoke test or CI to ensure no regression | No | Required before merging Windows support. |
| Run Windows manual smoke test | No | Launch, tray, floating surface, hook event, settings, quit. |
| Update this document with final gaps | No | Leave any known limitations explicit. |

## Known High-Risk Areas

- Transparent click-through windows behave differently across macOS and Windows.
- Unix sockets and Unix file permissions do not map directly to Windows.
- Hook commands must handle `.exe`, spaces in paths, and different environment
  variable syntax.
- Terminal focus and jump features are currently macOS-heavy.
- DPI scaling and multi-monitor coordinates need real Windows desktop testing.
- Packaging and updater metadata must not break existing macOS releases.

## Suggested MVP Boundary

The first Windows release should target:

- App starts and exits cleanly.
- Tray menu works.
- Floating surface renders and can be toggled.
- At least Claude Code and Codex hook install paths are verified, if those CLIs
  support the expected Windows configuration files.
- Bridge events reach AgentBro.
- Terminal focus suppression and terminal jump may be marked unsupported if they
  are not ready.
- Packaging produces a usable Windows installer or portable bundle.
