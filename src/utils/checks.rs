use poise::serenity_prelude::Guild;

use crate::Context;
use tracing::debug;

pub fn check_vc(guild: Guild, ctx: Context) -> bool {
    // Obtiene ids del canal de voz del bot y del usuario.
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|s| s.channel_id);
    let bot_channel_id = guild
        .voice_states
        .get(&ctx.serenity_context().cache.current_user_id())
        .and_then(|s| s.channel_id);

    // Verifica si el usuario y el bot estan en el mismo canal de voz.
    match channel_id {
        Some(channel) => {
            if let Some(bot_channel) = bot_channel_id {
                if channel != bot_channel {
                    return false;
                }
            }
            debug!("Los canales de voz coinciden.");
            true
        }
        None => {
            debug!("El usuario no esta en ningún canal de voz.");
            false
        }
    }
}
