use poise::serenity_prelude::UserId;

use crate::{Context, Error};

// Registra los comandos de slash en el servidor o globalmente.
#[poise::command(prefix_command, guild_only)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    if let Ok(id) = std::env::var("OWNER_ID") {
        if let Some(author) = ctx.author_member().await {
            if let Ok(owner_id) = id.parse::<u64>() {
                if UserId(owner_id) == author.user.id {
                    poise::builtins::register_application_commands_buttons(ctx).await?;
                    return Ok(());
                }
            }
        };
    };

    Ok(())
}
