# Stream Deck controller for Sonos

A Stream Deck plugin that can control Sonos (or any UPnP-compliant) speakers.

Very rough proof of concept at the moment.

## Developing

Using [cargo-make](https://github.com/sagiegurari/cargo-make):

- run `cargo make symlink` to symlink the plugin into your Stream Deck installation and the debug build into the plugin directory.
- run `cargo make kill-plugin` to build the plugin and kill the existing plugin instance so that Stream Deck restarts it.
- run `cargo make restart-sd` to gracefully terminate and then re-start the Stream Deck application, which will also update the plugin metadata (actions).

**These commands work on macOS only.**