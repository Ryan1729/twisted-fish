# Count your counters

A small game to figure out a good way to implement card countering machanics.

All cards come in one (or more) of `n` flavours.

On a player's turn they use a card to attempt to win. The attempt has a flavour. The cards each allow countering cards of a particular set of flavours. E.g. a card may counter cherry or blue raspberry cards, but be lemon flavoured. Counters can counter other counters.

If the attempt fails, each player draws back up to a full hand, and the next player goes.

Cards are dealt from the same deck with enough cards that at least one full hand left once all the players have a full hand.

For a longer game, the attempts can be attempts to gain points of the given flavour, and you need `k` differently flavoured points to win.

## WASM version

### Running locally

1. Install Rust via [rustup.rs](https://rustup.rs).

2. Install WebAssembly target:
```
rustup target add wasm32-unknown-unknown
```
3. Start dev server:
```
cargo run-wasm count-your-counters --release
```
4. Visit `http://localhost:8000` with your browser.

### Extra build options

These extra features can be adding then to the run-wasm `features` flag. Note that these are comma separated. For instance to activate `invariant-checking` and `logging` you can run:
```
cargo run-wasm count-your-counters --release --features invariant-checking,logging
```
## Desktop

The desktop version attempts to be cross platform. Only Linux and Windows have been tested at this time.

### Building/Running

1. Install Rust via [rustup.rs](https://rustup.rs).

2. Build via cargo
```
cargo build --release --bin count-your-counters
```
3. Run the executable
```
./target/release/count-your-counters
```

#### Linux specific notes

When building the Linux version, some additional packages may be needed to support building the [`alsa`](https://github.com/diwic/alsa-rs) library this program uses for sound, on Linux.
On Ubuntu, these packages can be installed as follows:

```
sudo apt install libasound2-dev pkg-config
```

If you don't care about sound you can build with the enabled-by-default `"non-web-sound"` feature flag turned off:

```
cargo build --release --bin count-your-counters --no-default-features
```

##### Wayland
As of this writing, [a library that this program uses does not allow specifying that parts of the screen need to be redrawn, on Wayland](https://github.com/john01dav/softbuffer/issues/9).
For now, you can run the executable with the `WINIT_UNIX_BACKEND` environment variable set to `"x11"` as a workaround.

```
WINIT_UNIX_BACKEND="x11" ./target/release/count-your-counters
```

## Feature flags

##### invariant-checking

With this enabled violations of certain invariants will result in a panic. These checks are disabled in default mode since (presumably) a player would prefer the game doing something weird to outright crashing.

##### logging

Enables additional generic logging. With this feature disabled, the logs will be compiled out, leaving no appreciable run-time overhead.

##### non-web-sound

Enables sound when not building for the web. On by default.

___

Source code licensed under Apache or MIT, at your option.
