# Twisted Fish

This is a single player only version of the card game [Twisted Fish](https://boardgamegeek.com/boardgame/24447/twisted-fish), which was published by the apparently defunct McNeill Designs ([web archive link](https://web.archive.org/web/20120215164041/http://www.mcneilldesigns.com/)).

As of this writing, the implementation is not complete. This project was started from [this template](https://github.com/Ryan1729/cross-platform-template).

Looking back at this after a while, I think finishing this needs a different approach than what I had been doing. In particular, it seems like a software design approach is needed to make it less complicated to support countering things with the Divine Intervention card. I think a good way to figure out how to do that properly is to experiemnt with a new game that doesn't have all the baggage.coplication of this one. The description of that game is [here](docs/Count-your-counters-README.md). That said, I am likely to work on other projects before coming back to work on this one.

## WASM version

### Running locally

1. Install Rust via [rustup.rs](https://rustup.rs).

2. Install WebAssembly target:
```
rustup target add wasm32-unknown-unknown
```
3. Start dev server:
```
cargo run-wasm twisted-fish --release
```
4. Visit `http://localhost:8000` with your browser.

### Extra build options

These extra features can be adding then to the run-wasm `features` flag. Note that these are comma separated. For instance to activate `invariant-checking` and `logging` you can run:
```
cargo run-wasm twisted-fish --release --features invariant-checking,logging
```
## Desktop

The desktop version attempts to be cross platform. Only Linux and Windows have been tested at this time.

### Building/Running

1. Install Rust via [rustup.rs](https://rustup.rs).

2. Build via cargo
```
cargo build --release --bin twisted-fish
```
3. Run the executable
```
./target/release/twisted-fish
```

#### Linux specific notes

When building the Linux version, some additional packages may be needed to support building the [`alsa`](https://github.com/diwic/alsa-rs) library this program uses for sound, on Linux.
On Ubuntu, these packages can be installed as follows:

```
sudo apt install libasound2-dev pkg-config
```

If you don't care about sound you can build with the enabled-by-default `"non-web-sound"` feature flag turned off:

```
cargo build --release --bin twisted-fish --no-default-features
```

##### Wayland
As of this writing, [a library that this program uses does not allow specifying that parts of the screen need to be redrawn, on Wayland](https://github.com/john01dav/softbuffer/issues/9).
For now, you can run the executable with the `WINIT_UNIX_BACKEND` environment variable set to `"x11"` as a workaround.

```
WINIT_UNIX_BACKEND="x11" ./target/release/twisted-fish
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
