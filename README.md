> **Warning**
> This project is being archived as I no longer have time to work on it and YouTube keeps making breaking changes that I can't keep up with. Feel free to fork or use as however you please.
<details>
  <summary>Original README</summary>
  > **Note**
  > This project is currently in maintenance-only mode, meaning that PRs may not be reviewed, no new features will be worked on and only critical bug fixes will be addressed.

  Discord music bot developed originally for a personal server, using [Serenity](https://github.com/serenity-rs/serenity), [Songbird](https://github.com/serenity-rs/songbird) and [Poise](https://github.com/kangalioo/poise).

  ## Environment variables

  | Name               | Description                     | Required |
  | ------------------ | ------------------------------- | -------- |
  | **DISCORD_TOKEN**  | Bot authorization token         | x        |
  | **CLIENT_ID**      | Bot ID (not used at the moment) | -        |
  | **OWNER_ID**       | Owner user ID                   | x        |
  | **YT_API_KEY**     | YouTube's API key               | -        |
  | **SPOTIFY_ID**     | Spotify's API ID                | x        |
  | **SPOTIFY_SECRET** | Spotify's API secret            | x        |

  ## Logging

  The bot uses the crate `tracing` for logging, which takes the environment variable `RUST_LOG` to set the logging level. Recommended value: `none,disco_rs=error`.

  ## Versioning

  This project follows the specifications established in [SemVer](https://semver.org).

  ## License

  Licensed under MIT, but used crates might have different licenses.
</details>
