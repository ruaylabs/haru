# haru (貼る)

Save an image from the system clipboard to a file. A small Rust alternative to
`pngpaste` for macOS and Linux.

## Install

Download `haru-<version>-macos-universal` (Apple Silicon and Intel) or
`haru-<version>-linux-x86_64` from this repository's GitHub Releases. Verify the
download against `SHA256SUMS`, then install it as `haru` in a directory on your PATH:

```sh
chmod +x haru-0.1.0-macos-universal
install -m 755 haru-0.1.0-macos-universal ~/.local/bin/haru
```

Create `~/.local/bin` first if necessary. On Linux, substitute
`haru-0.1.0-linux-x86_64`. Or install via Homebrew:

```sh
brew install ruaylabs/tap/haru
```

macOS release binaries are Developer ID signed and notarized;
Apple's notarization ticket is checked online (bare binaries cannot be stapled).

### NixOS

The Linux release binary dynamically links glibc, which NixOS does not provide in
standard FHS locations. Enable [nix-ld][nixld] system-wide to run it directly:

```nix
# configuration.nix
programs.nix-ld.enable = true;
```

If additional libraries are missing at runtime (for example `libwayland-client`
for Wayland), add them with `programs.nix-ld.libraries`. To avoid a system-wide
change, build from source instead; a local Cargo build links against the Nix
toolchain's glibc and runs without nix-ld:

```sh
nix-shell -p cargo rustc pkg-config
just build
```

[nixld]: https://github.com/Mic92/nix-ld

## Usage

Copy an image, then run:

```sh
haru screenshot.png
haru photo.jpg
haru - > clipboard.png
haru --help
haru --version
```

|Extension      |Format|
|---------------|------|
|`.png`         |PNG   |
|`.jpg`, `.jpeg`|JPEG  |
|`.gif`         |GIF   |
|`.tif`, `.tiff`|TIFF  |
|`.bmp`         |BMP   |

Extensions are case-insensitive. Missing extensions default to PNG; unknown
extensions emit a warning and also produce PNG, regardless of the filename.
`-` writes only PNG bytes to stdout; diagnostics go to stderr. JPEG discards
alpha without compositing. Existing output files are overwritten.

Exit codes: **0** success (including help/version), **1** usage or clipboard
error, **2** encoding or output error. No arguments prints usage and exits 1.

Linux requires an active graphical session: X11 or a Wayland compositor
supporting the data-control protocol. Headless sessions and unsupported Wayland
compositors cannot supply clipboard images. Copy actual image data, not just a
file path or text.

## Build and test

Use current stable Rust and Cargo:

```sh
cargo build --locked --release
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo install --locked --path .
```

Tests cover argument parsing, format detection, clipboard-data validation, and
encoding/decoding every supported format without requiring a display. For a
manual clipboard test, copy an image and run `cargo run -- test.png`, then check
`cargo run -- - > stdout.png` and the other output formats.

## Releases

Push a `v*` tag matching the package version to trigger
`.github/workflows/release.yml`. It builds a stripped Linux x86_64 executable and
a universal macOS executable, signs and notarizes macOS, and publishes both with
SHA-256 checksums. Grant this repository access to these organization secrets:

- `APPLE_CERTIFICATE`: base64-encoded Developer ID Application `.p12`
- `APPLE_CERTIFICATE_PASSWORD`: password for the `.p12`
- `APPLE_SIGNING_IDENTITY`: Developer ID Application signing identity
- `APPLE_ID`: Apple account email
- `APPLE_TEAM_ID`: Apple developer team ID
- `APPLE_PASSWORD`: app-specific password for notarization

The import step creates a temporary keychain and removes it after the job.

## License

MIT; see [LICENSE](LICENSE).
