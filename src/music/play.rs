use poise::{
    serenity_prelude::{Color, Member},
    ReplyHandle,
};
use regex::Regex;
use rspotify::{model::TrackId, prelude::BaseClient, ClientCredsSpotify, Credentials};
use songbird::input::{Input, Metadata};
use std::{collections::HashMap, time::Duration};
use tracing::{debug, error, info, warn};

use crate::{handlers, Context, Error};

enum Platform {
    Spotify,
    YouTube,
}

/// Busca el texto o url introducidos e intenta reproducirlos.
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL o palabras a buscar."] search: String,
) -> Result<(), Error> {
    info!("Recibida interacción del comando play.");

    // Obtiene ids del servidor.
    let guild = ctx.guild().expect("no guild");
    let guild_id = ctx.guild_id().unwrap();
    debug!("ID del servidor: {}", guild_id);

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
    let connect_to = match channel_id {
        Some(channel) => {
            if let Some(bot_channel) = bot_channel_id {
                if channel != bot_channel {
                    debug!("El bot y el usuario no están en el mismo canal de voz.");
                    ctx.say("No estas en el mismo canal de voz que el bot".to_string())
                        .await?;

                    return Ok(());
                }
            }
            debug!("Los canales de voz coinciden.");
            channel
        }
        None => {
            debug!("El usuario no esta en ningún canal de voz.");
            ctx.say("No estas en ningún canal de voz.".to_string())
                .await?;

            return Ok(());
        }
    };

    // Intenta conectar el bot al canal de voz.
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("lol")
        .clone();

    let member = match ctx.author_member().await {
        Some(member) => member,
        None => {
            error!("No se pudó obtener el miembro?");

            return Ok(());
        }
    };

    // Obtiene handler si existe o crea uno.
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        if let Err(e) = handler.join(connect_to).await {
            warn!("Ocurrió un problema al conectarse al canal de voz: {}", e)
        };

        handler.deafen(true).await?;

        ctx.defer().await?;

        if let (Some(source), Some(platform)) = get_source(&search).await {
            debug!("Se obtuvo correctamente la fuente de audio.");

            let mut metadata = source.metadata.clone();

            handler.enqueue_source(source);

            formatted_reply(ctx, metadata.take(), member.into_owned(), platform).await?;
        } else {
            ctx.say("No se pudo obtener la fuente de audio. Si buscaste con palabras, intenta introducir un link.").await?;
            return Ok(());
        };

        return Ok(());
    } else {
        let (handler_lock, success) = manager.join(guild_id, connect_to).await;

        if let Ok(_channel) = success {
            // No me gusta clonar esta variable acá, tiene que haber una solución más elegante...
            let event_handler_lock = handler_lock.clone();
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(
                songbird::Event::Periodic(Duration::from_secs(60), None),
                handlers::TimeoutHandler {
                    handler_lock: event_handler_lock,
                    elapsed: Default::default(),
                },
            );

            handler.deafen(true).await?;

            ctx.defer().await?;

            if let (Some(source), Some(platform)) = get_source(&search).await {
                debug!("Se obtuvo correctamente la fuente de audio.");

                let mut metadata = source.metadata.clone();

                handler.enqueue_source(source);

                formatted_reply(ctx, metadata.take(), member.into_owned(), platform).await?;
            } else {
                ctx.say("No se pudo obtener la fuente de audio. Si buscaste con palabras, intenta introducir un link.").await?;
                return Ok(());
            };
        }
    }

    Ok(())
}

// Obtiene la fuente de audio de un url de youtube o intenta buscar las palabras.
async fn get_source(query: &String) -> (Option<Input>, Option<Platform>) {
    debug!("get source");
    debug!("Query: {}", query);
    const SPOTIFY_REGEXP: &str = r"^https?://open.spotify.com/(?P<Type>track|user|artist|album|playlist)/(?P<ID>[a-zA-Z0-9]+)(/playlist/[a-zA-Z0-9]+|)|spotify:(track|user|artist|album):[a-zA-Z0-9]+(:playlist:[a-zA-Z0-9]+|)$";
    const YT_REGEXP: &str = r"^http(?:s?)://(?:www\.)?youtu(?:be\.com/watch\?v=|\.be/)([\w\-_]*)(&(amp;)?‌\u{200B}[\w\?‌\u{200B}=]*)?$";

    let spotify_re = Regex::new(SPOTIFY_REGEXP).unwrap();
    let yt_re = Regex::new(YT_REGEXP).unwrap();

    debug!("Empezando resolución de query...");
    if spotify_re.is_match(query) {
        debug!("Coincidencia con Spotify.");
        if let Some(spclient) = create_spotify_client().await {
            let caps = spotify_re.captures(query).unwrap();
            let caps_map: HashMap<&str, &str> = spotify_re
                .capture_names()
                .flatten()
                .filter_map(|n| Some((n, caps.name(n)?.as_str())))
                .collect();
            if caps_map["Type"] == "track" {
                debug!("1");
                let track_id = caps_map.get("ID").unwrap();
                let fmt_uri = format!("spotify:track:{}", track_id);
                let uri = TrackId::from_uri(&fmt_uri).unwrap();

                if let Ok(track) = spclient.track(uri).await {
                    debug!("2");
                    let name = track.name;
                    let artist = &track.artists[0].name;

                    let q = format!("{} - {}", name, artist);
                    debug!("{}", q);

                    match search_yt(&q).await {
                        Some(url) => match songbird::ytdl(url).await {
                            Ok(source) => {
                                debug!("Se obtuvo la fuente.");
                                return (Some(source), Some(Platform::Spotify));
                            }
                            Err(why) => {
                                error!("Error al obtener fuente de audio: {}", why);
                                return (None, None);
                            }
                        },
                        None => {
                            warn!("No se pudó buscar");
                            return (None, None);
                        }
                    }
                } else {
                    return (None, None);
                };
            }
        };
        (None, None)
    } else if yt_re.is_match(query) {
        debug!("Coincidió con YouTube");
        match songbird::ytdl(query).await {
            Ok(source) => (Some(source), Some(Platform::YouTube)),
            Err(why) => {
                error!("Error al obtener fuente de audio: {}", why);
                (None, None)
            }
        }
    } else {
        debug!("No coincidió con ninguna plataforma, buscando palabras...");
        match search_yt(query).await {
            Some(url) => match songbird::ytdl(url).await {
                Ok(source) => (Some(source), Some(Platform::YouTube)),
                Err(why) => {
                    error!("Error al obtener fuente de audio: {}", why);
                    (None, None)
                }
            },
            None => {
                warn!("No se pudó buscar");
                (None, None)
            }
        }
    }
}

// Formatea la respuesta en forma de embed.
async fn formatted_reply(
    ctx: Context<'_>,
    metadata: Metadata,
    member: Member,
    platform: Platform,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    let duration = format_duration(metadata.duration.unwrap());

    let mut title = metadata.title.unwrap_or_else(|| "N/A".to_string());
    title.truncate(128);

    ctx.send(|b| {
        b.embed(|e| {
            e.title(title)
                .url(metadata.source_url.unwrap_or_default())
                .thumbnail(metadata.thumbnail.unwrap_or_default())
                .author(|a| a.icon_url(member.face()).name("Agregó a la lista..."))
                .field(
                    "Canal",
                    metadata.artist.unwrap_or_else(|| "N/A".to_string()),
                    true,
                )
                .field("Duración", duration, true)
                .field(
                    "Plataforma",
                    match platform {
                        Platform::Spotify => {
                            "<:Spotify:1029496613481234534> ||(<:YouTube:1029496735816482866>)||"
                                .to_string()
                        }
                        Platform::YouTube => "<:YouTube:1029496735816482866>".to_string(),
                    },
                    true,
                )
                .color(Color::ORANGE)
        })
    })
    .await
}

// Formatea la duración en formato hh:mm:ss.
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs() % 60;
    let mins = (duration.as_secs() / 60) % 60;
    let hours = ((duration.as_secs() / 60) / 60) % 60;

    let duration: String = if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    };

    duration
}

// Busca en youtube un video cuyo titulo contenga las palabras envíadas a través del parametro.
// Habría que ver de rehacer esta función porque esta implementación es propensa a errores.
async fn search_yt(q: &String) -> Option<String> {
    const SYMBOL_REGEX: &str = r#"[\\~#%&*{}/:<>?|"-\.,]"#;
    let symbolr = Regex::new(SYMBOL_REGEX).unwrap();

    if let Ok(key) = std::env::var("YT_API_KEY") {
        match reqwest::get(format!("https://www.googleapis.com/youtube/v3/search?part=snippet&maxResults=5&q={}&type=video&key={}", q, key)).await {
            Ok(r) => {
                match r.json::<serde_json::Value>().await {
                    Ok(b) => {
                        if let Some(items) = b["items"].as_array() {
                            debug!("{:?}", items);
                            for i in items {
                                if let Some(title) = &i["snippet"]["title"].as_str() {
                                    let lower_q = q.to_lowercase();
                                    let clean_q = symbolr.replace_all(&lower_q, "");
                                    let search: Vec<&str> = clean_q.split_whitespace().collect();
                                    debug!("search: {:?}", search); 
                                    let search_len = search.len() / 2;
                                    debug!("search len: {:?}", search_len);
                                    let mut coincidences = 0;
                                    for word in search {
                                        let lower_title = title.to_lowercase();
                                        let clean_title = symbolr.replace_all(&lower_title, "");
                                        let title_vec: Vec<&str> = clean_title.split_whitespace().collect();
                                        debug!("clean title: {:?}", clean_title);
                                        debug!("clean title vec: {:?}", title_vec);
                                        for title_word in title_vec {
                                            if word == title_word {
                                                coincidences += 1;
                                                debug!("coincidences: {:?}", coincidences);
                                                if coincidences >= search_len {
                                                    debug!("COINCIDENCIA");
                                                    if let Some(id) = i["id"]["videoId"].as_str() {
                                                        let url = format!("https://youtube.com/watch?v={}", id);
                                                        debug!("URL COINCIDENTE: {:?}", url);
                                                        return Some(url)
                                                    } else {
                                                        error!("No se pudo encontrar la ID del video, revisar respuesta de la API.");
                                                        return None
                                                    };
                                                }
                                            }
                                        }
                                    }
                                };
                            }
                        } else {
                            error!("No se pudierón encontrar los items, es probable que la API haya cambiado o simplemente sea un error temporal.");
                            return None
                        };
                    }
                    Err(why) => error!("Error al formatear jason: {}", why)
                }
            }
            Err(why) => {
                error!("Error al hacer búsqueda por YouTube: {}", why)
            }
        }
    } else {
        warn!(
            "No se encuentra la clave de YouTube, las búsquedas por palabras no van a funcionar..."
        )
    };
    None
}

async fn create_spotify_client() -> Option<ClientCredsSpotify> {
    if let (Ok(id), Ok(secret)) = (std::env::var("SPOTIFY_ID"), std::env::var("SPOTIFY_SECRET")) {
        let creds = Credentials {
            id,
            secret: Some(secret),
        };
        let spotify = ClientCredsSpotify::new(creds);
        spotify.request_token().await.unwrap();

        return Some(spotify);
    }
    error!("No se pudo crear el cliente de Spotify.");
    None
}
