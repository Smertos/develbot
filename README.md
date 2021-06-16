# Devel Bot

## tf is this
bruh momento

## how do you launch this shit
* Copy `configs/config.example.toml` to `configs/config.toml`
* Edit `configs/config.toml`
* `cargo build`
* Run it LULW. I dunno how to properly deploy Rust apps FeelsDankMan

## whats next

who :tf: knows

Might work on something out of this list:
* Store past stream info, would need one of those:
  - sub to `stream started` event on eventsub
  - poll stream status myself
    
* Docker deployment
  - sucks ass
  - makes deployment that much easier
    
* Serve static
  - could provide pleblist interaction UI
  - if there was a point system, would be able to provide info about that to chatters
  - admin panel for simpler high-level bot management
    
* Pleblist
  - QueUp exists, could just use it `bruh`
  - tho if I were to do it, could do it, so it would support playing viewer submitted tracks and would fall back to multi-playlist song list arranged by broadcaster/bot admin otherwise
  - could integrate with Twitch's point system to use them for playlist suggestions/requests
  - could make a blacklist for tracks/chatters
  - request review for channel moderators?
    
* `~remind <user> <message>` command
* `~uptime` command
* `~last-stream` command
* `~alias <command-name> <cmd-with-args>` command

any suggestions are welcomed in issues
