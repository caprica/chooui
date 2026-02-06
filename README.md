# Chooui

A simple terminal-based media player.

Implemented using `Rust`, `ratatui` and `libmpv`.

## Status

This is early-development work-in-progress, mostly only the basics are done.

* Application errors are generally caught and handled, but not reported.
* The play queue is not implemented.
* UI is bare-bones, no on-screen keyboard affordances.

## Music Library Scanning

This application requires proper mp3/id3 tags to be defined for the media.

When scanning media, if the id3 tag cannot be read for any particular file,
that file is skipped.

## SQLite database

To connect to the database via the command line:

```shell
sqllite3 ./music.db
```

## Configuration

Use a configuration file in the standard place for your OS, for example on
Linux this a file located in `~/.config/chooui` and the file name is
`default-config.toml`.

Example:

```toml
version = 1
media_dirs = [
	"/disks/music1",
	"/disks/music2"
]
```

## Early UI

Very early UI screenshots, this will change drastically as development
continues:

![chooui](https://github.com/caprica/chooui/raw/master/etc/screenshot.png "chooui browser")

![chooui](https://github.com/caprica/chooui/raw/master/etc/screenshot2.png "chooui search")
