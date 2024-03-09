## Test

```bash
cargo test

# install tarpaulin for coverage report
cargo install cargo-tarpaulin

# generate coverage report
cargo tarpaulin --out Html

# for windows, generate coverage report and open it in the browser
cargo tarpaulin --out Html && start tarpaulin-report.html
```
