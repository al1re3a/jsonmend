# JSONMend

[![CI](https://github.com/al1re3a/jsonmend/actions/workflows/ci.yml/badge.svg)](https://github.com/al1re3a/jsonmend/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Turn the small, predictable JSON mistakes produced by models and command-line tools into valid-looking JSON without a runtime or network call.

JSONMend removes Markdown fences and comments, normalizes single-quoted strings, quotes simple bare keys, removes trailing commas, and closes unfinished objects or arrays. It is intentionally conservative: it does not invent missing values or silently reorder data.

```console
$ jsonmend examples/broken.json.txt --explain
- removed Markdown fence
- removed comments
- converted single-quoted strings
- quoted unquoted object keys
- closed unterminated containers
- removed trailing commas
{"request_id": "req_42", ...}
```

## Install

```console
cargo install --path .
```

Pipe content through it, pass a file, or use `--check` in CI. `--check` exits with status 2 when a repair would be required.

## Scope and safety

JSONMend is a repair filter, not a schema validator. Validate the result with your normal JSON parser and schema before executing commands or persisting important data. Strings are processed without evaluating their contents.

## Development

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

MIT licensed. Contributions are welcome.
