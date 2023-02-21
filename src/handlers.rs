use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use poise::{
    async_trait,
    serenity_prelude::{
        Activity, Context as SerenityContext, EventHandler, Mutex, OnlineStatus, Ready, VoiceState,
    },
};
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler};
use tracing::{debug, info};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: SerenityContext, _ready: Ready) {
        info!("Bot iniciado con éxito!");
        let activity = Activity::listening("/play");
        let status = OnlineStatus::DoNotDisturb;

        ctx.set_presence(Some(activity), status).await;
    }

    async fn voice_state_update(
        &self,
        ctx: SerenityContext,
        old: Option<VoiceState>,
        _new: VoiceState,
    ) {
        debug!("Estado de voz actualizado.");
        if let Some(state) = old {
            if let Some(user) = ctx.cache.user(state.user_id) {
                if user.bot {
                    return;
                }
            }

            let vc_id = state.channel_id.unwrap();
            let voice_channel = ctx.cache.channel(vc_id).unwrap();
            let guild = voice_channel.guild().unwrap().guild(&ctx.cache).unwrap();

            if let Some(guild) = ctx.cache.guild(guild.id) {
                if let Some(bot_vc_id) = guild
                    .voice_states
                    .get(&ctx.cache.current_user_id())
                    .and_then(|s| s.channel_id)
                {
                    if bot_vc_id == vc_id {
                        if let Some(vc) = guild.channels.get(&vc_id) {
                            if let Some(vcg) = vc.clone().guild() {
                                if let Ok(members) = vcg.members(&ctx.cache).await {
                                    if members.len() <= 1 {
                                        let manager =
                                            songbird::get(&ctx).await.expect("lol").clone();
                                        if let Some(handler_lock) = manager.get(guild.id) {
                                            let mut handler = handler_lock.lock().await;
                                            let _ = handler.leave().await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };
        }
    }
}

pub struct TimeoutHandler {
    pub handler_lock: Arc<Mutex<Call>>,
    pub elapsed: Arc<AtomicUsize>,
}

#[async_trait]
impl VoiceEventHandler for TimeoutHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        debug!("Disparado evento de inactividad!");

        let mut handler = self.handler_lock.lock().await;
        if handler.queue().is_empty() {
            debug!(
                "La lista esta vacía, agregando tiempo al contador de inactividad y verificando tiempo pasado..."
            );
            if (self.elapsed.fetch_add(1, Ordering::Relaxed) + 1) > 15 {
                debug!("Pasó el tiempo de inactividad, abandonando canal...");
                let _ = handler.leave().await;
            }
        } else {
            debug!("La lista no esta vacia, reiniciando contador de inactividad...");
            self.elapsed.store(0, Ordering::Relaxed);
        }

        None
    }
}
