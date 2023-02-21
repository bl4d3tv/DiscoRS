use crate::{utils, Context, Error};
use tracing::{debug, info};

/// Saltea la canción que se esta reproduciendo.
#[poise::command(slash_command, guild_only)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    info!("Recibida interacción del comando /stop.");

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

        queue.stop();

        ctx.say("Se paró la lista correctamente.").await?;
        return Ok(());
    }

    ctx.say("No hay ningúna lista de reproducción actualmente")
        .await?;

    Ok(())
}
