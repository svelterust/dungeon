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
cargo run --bin client --release -- -a dungeon.svelterust.com:9000
```
