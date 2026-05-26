# TeXas
> TeX Async Synchronization

TeXas is a proof-of-concept peer-to-peer LaTeX editor written in Rust. It combines a desktop UI built with `egui`, local `latexmk` compilation, and a custom text CRDT replicated over `libp2p` so multiple peers can edit the same document without a central server.

## Current capabilities

- Edit a shared LaTeX document in a native desktop window.
- Compile the document with 'latexmk' in app.
- Discover peers on the local network with mDNS and exchange CRDT operations over gossipsub.

## Requirements

- Rust toolchain with `cargo`
- A TeX distribution that provides `latexmk`
- A desktop environment capable of opening files with the system default app
  - Linux: `xdg-open`
  - macOS: `open`
  - Windows: `start`

## Running the application

```bash
cargo run
```

## Testing the application

The application can be tested by running two instances on the same computer. The instances will discover one another over mDns.

## Architecture

### App layer

The UI lives in [`src/app`]. It is responsible for:

- rendering the editor and build output panes
- saving the current buffer to `main.tex`
- invoking `latexmk`
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

## Limitations

TeXas is not a finished collaborative editor yet. The current implementation has some important constraints:

- Only one document is supported, and it is hardcoded to `main.tex`.
- Collaboration is currently aimed at peers on the same local network discovered through mDNS.
- There is no access control, persistence layer for collaborative sessions, or document/session management yet.

## Development

Run the test suite with:

```bash
cargo test
```
