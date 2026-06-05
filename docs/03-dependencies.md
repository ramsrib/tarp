# 03 — Dependencies

External dependency audit. Focus is on the **git dependencies** (the ones that
pin us to warpdotdev infra) and the 5 crates `about.toml` wrongly claims aren't
open-sourced.

## Totals

- `Cargo.lock`: **1,507** resolved packages.
- Direct external **git dependencies** in root `Cargo.toml`: **22 lines / 16 distinct repos**.
- More warpdotdev forks are pulled **transitively** (observed during the build fetch:
  `difflib`, `rust-email_address`, `uneval`, …) — i.e. the dependency on the
  warpdotdev org is deeper than the 16 direct repos.

## Git dependencies (direct)

| Repo | Crates | Nature | Notes |
|---|---|---|---|
| `servo/core-foundation-rs` | `core-foundation`, `-sys`, `core-graphics`, `core-text` | Upstream (servo) | Pinned to a rev; explicitly allow-listed in `deny.toml`. Not warpdotdev. |
| `warpdotdev/winit` | `winit` | Fork of OSS | Windowing. |
| `warpdotdev/font-kit` | `font-kit` | Fork of OSS | Font loading. |
| `warpdotdev/pathfinder` | `pathfinder_simd` | Fork of OSS | Vector/SIMD. |
| `warpdotdev/rust-objc` | `objc` | Fork of OSS | macOS ObjC bridge. |
| `warpdotdev/vte` | `vte` | Fork of OSS | Terminal escape parser (core!). |
| `warpdotdev/yaml-rust` | `yaml-rust` | Fork of OSS | YAML. |
| `warpdotdev/notify` | `notify-debouncer-full` | Fork of OSS | File watching. |
| `warpdotdev/tink-rust` | `tink-core/-proto/-hybrid` | Fork of OSS | Crypto (Google Tink). |
| `warpdotdev/jemallocator` | `tikv-jemallocator`, `-sys` | Fork of OSS | Allocator. |
| `warpdotdev/mermaid-to-svg` | `mermaid_to_svg` | warpdotdev | Mermaid diagram rendering (AI/markdown output). |
| `warpdotdev/command-corrections` | `command-corrections` | warpdotdev (MIT) | See §"the 5" — **keep**. |
| `warpdotdev/command-signatures` | `warp-command-signatures` | warpdotdev (MIT) | See §"the 5" — **keep**. |
| `warpdotdev/workflows` | `warp-workflows` | warpdotdev (Apache-2.0) | See §"the 5" — **keep**. |
| `warpdotdev/session-sharing-protocol` | `session-sharing-protocol` | warpdotdev (AGPL) | See §"the 5" — **remove**. |
| `warpdotdev/warp-proto-apis` | `warp_multi_agent_api` | warpdotdev (AGPL) | See §"the 5" — **remove**. |

A `[patch]` stanza exists for local dev of `warp-proto-apis` (commented out).

## The 5 "not-yet-open-sourced" crates — actually public

`about.toml` carries a stale comment claiming these "are not part of this
workspace and do not have explicit licenses yet (they will be open-sourced
soon)." **Verified false on 2026-06-05:** all five return HTTP 200, are
`git ls-remote`-cloneable anonymously, and carry SPDX licenses. **None block the
build.** Only 2 of 5 relate to features we're removing.

| Crate (repo) | License | What it does | Wired into (`.rs` files) | Verdict |
|---|---|---|---|---|
| **command-corrections** | MIT | `thefuck`-style "did you mean" — failed command + exit code + shell → suggested fix | `warp_core`, `warp_terminal`, `warp_features`, `app`, `integration` (13) | **Keep** — terminal UX |
| **warp-command-signatures** (command-signatures) | MIT | Command spec/signatures for completions (args, templates, icons; `signature_by_name("kubectl")`); Fig-spec-like | `warp_completer` (17), `app` (1) | **Keep** — completions |
| **warp-workflows** (workflows) | Apache-2.0 | "Workflows" = saved/parameterized command snippets | `cloud_object_models`, `app`, `integration` (5) | **Keep**, but entangled with cloud layer |
| **session-sharing-protocol** | AGPL-3.0 | Real-time session-sharing/collab protocol types (Guest, Role, ACL, sharer/viewer) | **`app` (66)**, `cloud_object_*` (4), `warp_terminal` (2) | **Remove** — cloud/collab |
| **warp_multi_agent_api** (warp-proto-apis) | AGPL-3.0 | Protobuf API for AI multi-agent conversations (Message, ConversationData, response_event, file-diff results) | **`app` (56)**, `ai` (11), `persistence` (2), `integration` (1) | **Remove** — AI backbone |

## Decisions needed (dependency strategy)

1. **warpdotdev OSS forks** (`vte`, `winit`, `font-kit`, etc.): keep pinned for v1
   (lowest risk), or re-point to upstream / vendor under a Tarp org later. `vte` in
   particular is core terminal parsing — a vendored fork is a maintenance burden but
   also a place where Warp may have terminal-specific patches we need.
2. **`deny.toml`** currently `allow-org = { github = ["warpdotdev"] }` — for a hard
   fork we'd eventually want to drop blanket trust of the warpdotdev org and
   allow-list specific repos (or our own fork org).
3. **Transitive warpdotdev forks** (`difflib`, `uneval`, …) need enumerating from
   `Cargo.lock` if we want zero warpdotdev dependency long-term.

See [`04-licensing.md`](04-licensing.md) for the license-config fixes these imply.
