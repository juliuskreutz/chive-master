# ChiveMaster

- [Description](#description)
- [Hosting](#hosting)
- [Building](#building)

## Description

A discord bot, that lets users register with their Honkai Star Rail uid to participate in a chive (achievement) hunting leaderboard.

## Hosting

Just grab the latest release executable [here](https://github.com/juliuskreutz/chive-master/releases/latest). (Currently only x86_64-linux is supported). Put a `.env` file in the same folder that looks like this (replacing the token with your bot token). You might need to install sqlite3

```
DISCORD_TOKEN=token
DATABASE_URL=sqlite:db.sqlite
```

## Building

If you want to build the bot yourself you have to follow these steps. \
(This process uses these packages under the hood `cc pkg-config libsqlite3 libssl`. So you might need to install them)

- Clone this repository
- Change `.env` accordingly
- Install rust (https://www.rust-lang.org/tools/install)
- Install sqlx `cargo install sqlx-cli`
- Create database `sqlx db create`
- Migrate database `sqlx migrate run`
- Finally you can run the bot `cargo run`
