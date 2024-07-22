# Vara Pebble Game Contract

### 🏗️ Building

```sh
cargo build --release
```

### ✅ Testing

Run all tests, except `gclient` ones:
```sh
cargo t --workspace -- --skip gclient
```

Run all tests:
```sh
# Download the node binary.
cargo xtask node
cargo t --workspace