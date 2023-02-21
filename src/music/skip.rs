use crate::{utils, Context, Error};
use tracing::{debug, info};

/// Saltea la canción que se esta reproduciendo.
#[poise::command(slash_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    info!("Recibida interacción del comando /skip.");

    // Obtiene ids del servidor.
    let guild_id = ctx.guild_id().unwrap();
    debug!("ID del servidor: {}", guild_id);

    if !utils::checks::check_vc(ctx.guild().unwrap(), ctx) {
        ctx.say("No estas en el mismo canal de voz que el bot.")
            .await?;
        return Ok(());
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("lol")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let queue = handler.queue();

        match queue.skip() {
            Ok(()) => {
                ctx.say("Se salteo correctamente la canción actual.")
                    .await?;

                return Ok(());
            }
            Err(why) => {
                debug!("Error al skipear: {}", why);

                ctx.say("Ocurrió un error al saltear la canción.").await?;

                return Ok(());
            }
        }
    }

    Ok(())
}
