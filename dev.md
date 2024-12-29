# Dev Notes

## Coverage

```bash
# install tarpaulin for coverage report
cargo install cargo-tarpaulin

# generate coverage report
cargo tarpaulin --out Html

# for windows, generate coverage report and open it in the browser
cargo tarpaulin --out Html && start tarpaulin-report.html
```

## Bench

```bash
cargo bench
```

## Profile

Linux only.

```bash
cargo install flamegraph

cargo flamegraph --bench json_lexer -- --bench
```

## Publish

GitHub Actions will be triggered when a new tag with pattern `v*.*.*` is pushed.
See [`.github/workflows/release.yml`](.github/workflows/release.yml) for more details.
