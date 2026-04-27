# Contributing to nahook-rust

Thanks for considering a contribution! A few important things to know first.

## Source of truth

This repository is a **subtree-split mirror** of the Rust SDK from our private monorepo `getnahook/nahook`. PRs filed directly here **cannot be merged** — the next subtree-push from the monorepo will force-overwrite this branch.

## What we welcome

- **Bug reports** — open a GitHub issue with: reproduction steps, crate version, `rustc --version --verbose`, OS, and your `Cargo.toml` snippet.
- **Feature requests** — open an issue describing the use case and the API surface you'd want.
- **Small code suggestions** — paste a snippet in an issue and describe intent; we'll port it into the monorepo and credit you in the resulting commit.
- **Substantial patches** — email `support@nahook.com` first; we'll either discuss read access to the monorepo or hand-port your change with credit.

## Local development

```bash
git clone https://github.com/getnahook/nahook-rust
cd nahook-rust
cargo build
cargo test                          # ~100 tests across 9 binaries
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo package --no-verify           # sanity-check the publish path
```

`Cargo.toml` declares `edition = "2021"`. SDK targets stable Rust.

### Code style

- `cargo fmt --check` must be clean (CI enforces)
- `cargo clippy --all-targets -- -D warnings` must be clean (CI enforces)
- `Cargo.lock` IS committed (modern library convention since 1.62 — gives reproducible contributor builds; auto-ignored when used as a dep)

## License

By contributing, you agree your changes are released under the [MIT License](LICENSE).
