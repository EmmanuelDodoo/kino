<div align="center">
    <img src="resources/logo/logo.png" width="140px" alt="Kino log" />

# Kino

A local movie and TV show library manager and player with support for curated collections.

<img src="resources/images/showcase/movies.png" width="400px" alt="Movies image"/>
<img src="resources/images/showcase/season.png" width="400px" alt="Season image"/>

See the [showcase directory](resources/images/showcase) for more images.

</div>

## Features

- Scan and index any number of local directories for media files

- Fast search across your library with filtering and sorting options

- Automatic collections with customizable rules and smart population

- Playlist creation, temporary selection mode, and playlist saving

- Comments, notes, and metadata management for videos

- Wishlist support for tracking media to watch later

- Fetch extra metadata from use selected sources

- Fully customizable keybindings for actions throughout the app

Take a look at [the documentation](/resources/docs/README.md) for more.

## Installation

Kino currently provides pre-built packages for the Windows and Linux (via deb and flatpak).
The latest releases can be found on the [project's Github Releases page.](https://github.com/EmmanuelDodoo/kino/releases)

### macOS

Pre-built macOS packages are not currently provided.

While Kino is designed to be cross-platform, I do not currently have access to macOS hardware for testing or validating packaged releases. Rather than distributing binaries that cannot be properly tested, macOS users are encouraged to build Kino from source.

Contributions and testing from macOS users are always welcome.

## Building from Source

### Prerequisites

Kino uses **GStreamer 1.26.6** for media playback. You'll need to install both the runtime files to build this project. The official instructions for that can be found here [https://gstreamer.freedesktop.org/download/]()

Kino can then be built like so:

```
git clone https://github.com/EmmanuelDodoo/kino.git
cd kino
cargo build --release
./target/release/kino
```

## Acknowledgement

Much thanks to

- [jazzfool](https://github.com/jazzfool) for both their work on [iced_video_player](https://github.com/jazzfool/iced_video_player) and [jangal](https://github.com/jazzfool/jangal) which provided some inspiration.

- [Iced](https://github.com/iced-rs/iced)

## Feedback

Feel free to leave any feedback via an issue or pull request.
