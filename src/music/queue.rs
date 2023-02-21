use tracing::{debug, info};

use crate::{Context, Error};

/// Muestra la lista de reproducción del momento.
#[poise::command(slash_command, guild_only)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    info!("Recibida interacción del comando /queue.");

    // Obtiene ids del servidor.
    let guild_id = ctx.guild_id().unwrap();
    debug!("ID del servidor: {}", guild_id);

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("lol")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let queue = handler.queue().clone();
        debug!("Cosas en la lista: {}", queue.len());

        if !queue.is_empty() {
            let mut track_counter = 0;

            let mut formatted_queue = String::new();

            queue.modify_queue(|q| {
                for t in q.iter() {
                    track_counter += 1;
                    formatted_queue = format!(
                        "{}\n{}. {}",
                        formatted_queue,
                        track_counter,
                        t.metadata().title.as_ref().unwrap()
                    );
                }
            });

            ctx.say(formatted_queue).await?;

            return Ok(());
        } else {
            ctx.say("No hay cosas en la lista").await?;

            return Ok(());
        }
    }

    Ok(())
}
