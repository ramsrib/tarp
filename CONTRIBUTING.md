# Contributing to Tarp

Thanks for your interest in Tarp! Tarp is a community fork of Warp's open-source
terminal with the AI, cloud, account, and code-editor layers removed — the goal is
a fast, plain, local terminal. This guide covers how to build, what fits Tarp's
scope, and how to get changes reviewed.

> Tarp is an independent fork and is not affiliated with Warp / Denver Technologies.

## Project scope (please read before proposing features)

Tarp deliberately **does not** include AI/agents, cloud sync, accounts/sign-in, or
a built-in code editor. Contributions that add those back are out of scope. In
scope: the terminal itself — rendering, input, blocks, completions, command
corrections, workflows, themes, SSH, shell integration, performance, platform
support, and bug fixes.

When in doubt, open an issue to discuss before writing a large change.

## Building and running

See [`BUILD.md`](BUILD.md) for prerequisites (notably the macOS Metal Toolchain)
and full instructions.

```sh
./script/run         # build + bundle + launch Tarp
cargo build --bin tarp --features gui
./script/presubmit   # fmt + clippy + tests (run before pushing)
```

## Development workflow

1. Fork the repo and create a branch off `main`.
2. Make your change. Keep diffs focused.
3. Run `./script/presubmit` (or at least `./script/format` + `cargo build`).
4. Open a PR against `main` using the PR template. Include a screenshot or short
   recording for user-visible changes.

### Code style

- Rust is formatted with `rustfmt` (`./script/format`); CI checks `cargo fmt --check`.
- Clippy should be clean for code you touch (`cargo clippy --bin tarp --features gui`).
- Match the surrounding code's conventions.

### A note on upstream

Tarp tracks upstream Warp selectively (see
[`docs/08-upstream-sync.md`](docs/08-upstream-sync.md)). To keep that sustainable,
**avoid gratuitous changes to the terminal-core crates** (`warpui*`,
`warp_terminal`, `warp_core`, `editor`, `command`, `warp_completer`, `vim`,
`syntax_tree`, `markdown_parser`) — keeping them close to upstream means we can
still cherry-pick upstream fixes. Larger structural work belongs in the `app/` layer.

## Reporting bugs and security issues

- **Bugs / features:** open an issue using the templates.
- **Security vulnerabilities:** do **not** open a public issue — see
  [`SECURITY.md`](SECURITY.md).

## Code of Conduct

Be respectful and empathetic. See [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Licensing

By contributing, you agree your contributions are licensed under the same terms as
the files you modify — AGPL-3.0 for most of the repo, MIT for `warpui`/`warpui_core`
(see [`README.md`](README.md#licensing) and [`NOTICE`](NOTICE)).
