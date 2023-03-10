use poise::serenity_prelude::Color;

use crate::{Context, Error};

/// Informaci贸n del bot
#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("VERGEN_GIT_SHA");
    let build_timestamp = env!("VERGEN_BUILD_TIMESTAMP");
    let build_id = env!("DISCO_GITHUB_RUN_NUMBER");

    let footer_icon = String::new();
    ctx.send(|b| {
        b.embed(|e| {
            e.title("DiscoRS 馃")
                .thumbnail(ctx.serenity_context().cache.current_user().face())
                .color(Color::ORANGE)
                .url("https://github.com/bl4d3tv/DiscoRS")
                .footer(|f| f.text("Desarrollado con 鉂わ笍 por BL4D3").icon_url(footer_icon))
                .field("Versi贸n", version.to_string(), false)
                .field("Hash de confirmaci贸n", git_hash.to_string(), false)
                .field("Tiempo de compilaci贸n", build_timestamp.to_string(), false)
                .field("ID de compilaci贸n", build_id, false)
        })
    })
    .await?;

    Ok(())
}
