# Kino Documentation

## Overview

This document covers advanced usage, tips, configuration and customization options for Kino.

## Tips

### General

- Click the poster image in any layout to start playback
- Click a video title in a playlist to jump to it
- Hovering over status toasts prevents them from disappearing
- Year-based filtering currently supports release year only
- Each sort category can be set independently to ascending or descending
- Multiple sorts can be applied; their order is shown by the number indicator
- Click on the show/season within the season/episode page to navigate there
- Loaded subtitles are saved and reused automatically
- Kino supports these subtitle extension types: `srt`, `ass`, `ssa`, `vtt`, `sub`, `sbv`, `ttml`, `dfxp`, `lrc`
- Subtitle files directly neighboring a video file and sharing the same name are automatically associated with that video.
- In the video player, double clicking enters full-screen mode, right clicking opens the video config dialog
- Ctrl + Clicking on a media item starts building a playlist
- Navigating to other pages while building a playlist maintains the selected media
- Timestamps can be added to video comments with the formats @now, @m:ss, @h:mm:ss (eg @12:04). 

### Searching

- Searches look for matches in media titles, overviews and tags
- Filter by media type in search using `type:`.
  - Eg "movie: Suzume" will only return results in movies
- Searches support logical `AND` and `OR`

### Settings

- Toggles can be enabled or disabled by clicking their labels
- Removed media can be restored by rescanning directories (if enabled in settings)
- In Settings, click a directory’s media type to toggle it.
  - Saving will remove all media records for that directory
- On the Settings page, click an individual keybinding to remove it

## Folder Scanning

Directories in Kino are assigned a single media type: movies or TV shows, never both. Each type expects its own folder structure. The directory type can be set from the Settings menu.

Supported video extensions:

`m4v`, `mkv`, `mov`, `mp4`, `mpgeg`, `mpg`, `webm`

### Regex Patterns

As part of the scanning process, video and folder names are sanitized using multiple regex patterns. I've tried my best to account for most common scenarios. Below are the patterns and when they're used.

#### Movies/Shows

- `^(.*?)(?=\d{3,4}p)|^.*$`: Find name then,
- `^(.*)(?=(\(|\[|\.)\d{4}(\)|\]|\.)(?!.*(\(|\[|\.)\d{4}(\)|\]|\.)))|^.*`: Find name and finally
- `[^a-zA-Z\d]+`: Replace matches with ' '

#### Seasons

- `(?<=[s|S])\s?\d{1,3}|(?<=[^a-zA-Z][s|S][e|E][a|A][s|S]|[o|O][n|N])\s?\d{1,3}`: Find season number

### Episodes

- `(?<=[e|E])\s?\d{1,3}|(?<=[e|E][p|P][i|I][s|S][o|O][d|D][e|E])\s?\d{1,3}|(?<=\dx)\s?\d{1,3}|^\d{1,3}(?=\s)`: Find episode number

### Movies

Movie folders consist of supported video files within a directory. Optionally, videos may be nested up to `N` folder levels deep, where `N` is configurable in the settings.

Example structure with `N=1`

```
── Movies
   ├── Blended.mp4
   ├── Oldies
        └── The Princess Bride.mkv
   └── Isle of Dogs.mov
```

### Shows

TV show directories are expected to follow a `Show → Season → Episode folder` structure. This structure is not configurable. A single TV directory may contain any number of shows.

```
── Shows
    ├── Good Omens
        ├── Season 1
            └── Episode 3
    ├── House
        ├── Season 7
            └── Episode 18

```

## Configuration

### Config File

Most application settings can be saved in a `$USER/config.toml` file. The corresponding `$USER` directory for Linux, Windows and macOS can be found [here](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.config_local_dir).

### Config Options

All available configuration options are listed in [the example config file](config.toml). Any fields not explicitly set will revert to their default values when the application exits.

Additionally, defaults for page actions can be disabled by adding `defaults = false` to the corresponding section header..

Example

```
[keybindings.player]
defaults = false
```

disables the default keybindings for the video player.

All available actions can be found in the [actions file](actions.md).

### Application Themes

Below are the available application Themes. Most of these are derived from [Daisy Ui](https://daisyui.com/)

- Abyss,

- Autumn,

- Black,

- Ember,

- Fantasy,

- Luxury,

- Winter,
