# dungeon

This game intentionally avoids bloated libraries. Handcrafted code is more delightful.

```sh
nix develop
cargo run --bin server # run server
cargo run --bin client # player 1
cargo run --bin client # player 2
```

## Connect to public server

```sh
cargo run --bin client --release -- -a 66.241.125.19:8080
```

## Web frontend

```
cargo build --release --bin client --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/client.wasm web/
cd web/ && python -m http.server
```
