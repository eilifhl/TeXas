# TeXas
> TeX Async Synchronization

TeXas is a proof-of-concept peer-to-peer LaTeX editor written in Rust. It combines a desktop UI built with `egui`, local compilation through the `tectonic` Rust library, and a custom text CRDT replicated over `libp2p` so multiple peers can edit the same document without a central server.

## Current capabilities

- Edit a shared LaTeX document in a native desktop window.
- Compile the document in app through the embedded `tectonic` engine.
- Discover peers on the local network with mDNS and exchange CRDT operations over gossipsub.

## Requirements

- Rust toolchain with `cargo`
- Tectonic native dependencies
  - macOS with Homebrew: `brew install pkgconf icu4c`
  - Linux: install the ICU development package for your distribution, for example `libicu-dev` on Debian/Ubuntu
- A desktop environment capable of opening files with the system default app
  - Linux: `xdg-open`
  - macOS: `open`
  - Windows: `start`

## Running the application

```bash
cargo run
```


If you experience issues running under Wayland, try running under Xwayland:
```bash
env -u WAYLAND_DISPLAY cargo run
```

## Testing the application

The application can be tested by running two instances on the same computer. The instances will discover one another over mDns.

## Dependencies

- `anyhow`: used for ergonomic error handling and propagating application errors.
- `eframe`: provides the native desktop application framework used to run the GUI.
- `egui`: used to build the editor interface and the rest of the user interface.
- `tectonic`: used to compile LaTeX source into PDF directly inside the application.
- `uuid`: used to generate unique identifiers for replicas and messages.
- `serde`: used to serialize and deserialize the project's data structures.
- `bincode`: used for compact binary encoding of messages sent over the network.
- `tokio`: provides the async runtime used for networking and background tasks.
- `libp2p`: used for peer-to-peer communication, including TCP transport, mDNS peer discovery, Noise encryption, Yamux multiplexing, and gossipsub messaging.

## Architecture

### App layer

The UI lives in [`src/app`]. It is responsible for:

- rendering the editor and build output panes
- saving the current buffer to `main.tex`
- invoking the embedded `tectonic` compiler
- opening the most recent PDF
- polling network events and applying remote edits

### Networking

The networking code lives in [`src/network`]. TeXas currently uses:

- `libp2p` TCP transport with Noise and Yamux
- mDNS for local peer discovery
- gossipsub for broadcasting document operations
- a single hardcoded topic: `texas/document/main`

### CRDT

The CRDT implementation lives in [`src/crdt`]. It is a text sequence CRDT with:

- per-replica operation and element identifiers
- tombstone-based deletes
- buffering for inserts that arrive before their dependencies

More info about the CRDT [here](CRDT.md).

## Limitations

TeXas is not a finished collaborative editor yet. The current implementation has some important constraints:

- Only one document is supported, and it is hardcoded to `main.tex`.
- Collaboration is currently aimed at peers on the same local network discovered through mDNS.
- There is no access control, persistence layer for collaborative sessions, or document/session management yet.
- Undo and redo are not implemented.

## Development

Run the test suite with:

```bash
cargo test
```

## Inspiration and resources
We have used several different sources as inspiration and guidance to our implementation:
- https://martin.kleppmann.com/papers/interleaving-papoc19.pdf
- https://www.inkandswitch.com/peritext/static/cscw-publication.pdf
- https://www.youtube.com/watch?v=x7drE24geUw
- https://arxiv.org/pdf/2310.18220
