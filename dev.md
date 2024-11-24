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
