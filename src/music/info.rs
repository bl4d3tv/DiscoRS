use poise::serenity_prelude::Color;

use crate::{Context, Error};

/// Información del bot
#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("VERGEN_GIT_SHA");
    let build_timestamp = env!("VERGEN_BUILD_TIMESTAMP");
    let build_id = env!("DISCO_GITHUB_RUN_NUMBER");

    let footer_icon = String::new();
    ctx.send(|b| {
        b.embed(|e| {
            e.title("DiscoRS 🦀")
                .thumbnail(ctx.serenity_context().cache.current_user().face())
                .color(Color::ORANGE)
                .footer(|f| f.text("Desarrollado con ❤️ por BL4D3").icon_url(footer_icon))
                .field("Versión", version.to_string(), false)
                .field("Hash de confirmación", git_hash.to_string(), false)
                .field("Tiempo de compilación", build_timestamp.to_string(), false)
                .field("ID de compilación", build_id, false)
        })
    })
    .await?;

    Ok(())
}
