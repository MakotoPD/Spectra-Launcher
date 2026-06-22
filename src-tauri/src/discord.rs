//! Discord Rich Presence (privacy-gated via settings.discord_rpc).
//!
//! The client connects lazily and is kept in `AppState`. Set the presence while
//! a game is running and clear it on exit. All failures are swallowed — Discord
//! not running must never affect launching.
//!
//! NOTE: the Discord Application ID comes from `DISCORD_CLIENT_ID` in `.env`
//! (embedded at build time by build.rs). When empty, RPC simply won't connect.

use discord_rich_presence::activity::Activity;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

use crate::AppState;

// Injected at build time from .env (see build.rs); empty when unset → no RPC.
const CLIENT_ID: &str = env!("DISCORD_CLIENT_ID");

/// Recomputes the Discord presence from every instance that's currently running
/// (`state.discord_playing`). This keeps the presence correct with multiple
/// simultaneous instances: it never shows a stale/overwritten one, and the first
/// instance to exit doesn't wipe the presence while others are still running.
///
/// - 0 running  → clear presence
/// - 1 running  → "Playing <name>" / "Minecraft <mc>"
/// - N running  → "Playing N instances" / "<name> · Minecraft <mc>" (the latest)
pub fn update_presence(state: &AppState) {
    let snapshot: Vec<(String, String)> = match state.discord_playing.lock() {
        Ok(map) => map.values().cloned().collect(),
        Err(_) => return,
    };

    let Ok(mut guard) = state.discord.lock() else { return };

    // Nothing playing → clear (but only touch the client if we have one).
    if snapshot.is_empty() {
        if let Some(client) = guard.as_mut() {
            let _ = client.clear_activity();
        }
        return;
    }

    // Connect lazily on first use.
    if guard.is_none() {
        if let Ok(mut client) = DiscordIpcClient::new(CLIENT_ID) {
            if client.connect().is_ok() {
                *guard = Some(client);
            }
        }
    }

    if let Some(client) = guard.as_mut() {
        let (details, line) = if snapshot.len() == 1 {
            let (name, mc) = &snapshot[0];
            (format!("Playing {name}"), format!("Minecraft {mc}"))
        } else {
            let (name, mc) = &snapshot[0];
            (
                format!("Playing {} instances", snapshot.len()),
                format!("{name} · Minecraft {mc}"),
            )
        };
        let _ = client.set_activity(Activity::new().details(&details).state(&line));
    }
}
