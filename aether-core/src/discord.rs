use azalea::app::{Plugin, Update};
use azalea::chat::{ChatPacket, ChatPacketKind, ChatReceivedEvent, SendChatKindEvent};
use azalea::ecs::prelude::*;
use azalea::entity::metadata::Player;
use azalea::entity::LocalEntity;
use azalea::prelude::*;
use azalea::protocol::packets::game::clientbound_system_chat_packet::ClientboundSystemChatPacket;
use bevy_discord::bot::events::BMessage;
use bevy_discord::bot::serenity::all::ChannelId;
use bevy_discord::bot::DiscordBotRes;
use bevy_discord::runtime::tokio_runtime;
use serde_json::json;
use std::sync::Arc;
use tracing::error;

use crate::chat::handle_chat;
use crate::config::Bot;

pub struct AetherDiscordPlugin;

/// Component present when `chat_relay` is turned on
#[derive(Component)]
pub struct DiscordChatRelay {
    pub channel_id: ChannelId,
    pub entity: Entity
}

/// Component present when `channel_id` is passed in `config.json`
#[derive(Component)]
pub struct DiscordChannelId {
    pub channel_id: ChannelId,
    pub entity: Entity
}

impl Plugin for AetherDiscordPlugin {
    fn build(&self, app: &mut azalea::app::App) {
        app.add_systems(
            Update,
            (
                handle_chat_relay,
                handle_discord_bridge,
                handle_disocrd_channel_id,
            )
                .chain()
                .after(handle_chat),
        );
    }
}

#[allow(clippy::complexity)]
fn handle_chat_relay(
    mut events: EventReader<ChatReceivedEvent>,
    query: Query<(&DiscordChatRelay, &Bot), (With<Player>, With<LocalEntity>, With<DiscordChatRelay>)>,
    discord_bot_res: Res<DiscordBotRes>,
) {
    for chat_received_packet in events.read() {
        for (discord_chat_relay, state) in query.iter() {
            let (sender, message) = chat_received_packet.packet.split_sender_and_content();
            let sender = sender.unwrap_or("Server".to_string());

            if sender != state.username
                && let Some(http) = discord_bot_res.get_http()
            {
                let channel_id_clone = discord_chat_relay.channel_id.clone();
                tokio_runtime().spawn(async move {
                    if http
                        .send_message(
                            channel_id_clone,
                            Vec::new(),
                            &json!({
                                "content": format!("{} -> {}", sender, message)
                            }),
                        )
                        .await
                        .is_err()
                    {
                        error!("Unable to send message on discord");
                    }
                });
            }
        }
    }
}

#[allow(clippy::complexity)]
fn handle_discord_bridge(
    mut events: EventReader<BMessage>,
    query: Query<&DiscordChatRelay, (With<Player>, With<LocalEntity>, With<DiscordChatRelay>)>,
    mut send_chat_kind_event: EventWriter<SendChatKindEvent>,
) {
    for BMessage {
        ctx: _,
        new_message,
    } in events.read()
    {
        for discord_chat_relay in query.iter() {
            if !new_message.author.bot && &new_message.channel_id == &discord_chat_relay.channel_id {
                send_chat_kind_event.send(SendChatKindEvent {
                    entity: discord_chat_relay.entity,
                    content: new_message.content.to_owned(),
                    kind: ChatPacketKind::Message,
                });
            }
        }
    }
}

#[allow(clippy::complexity)]
fn handle_disocrd_channel_id(
    mut events: EventReader<BMessage>,
    query: Query<&DiscordChannelId, (With<Player>, With<LocalEntity>, With<DiscordChannelId>)>,
    mut chat_received_event: EventWriter<ChatReceivedEvent>,
) {
    for BMessage {
        ctx: _,
        new_message,
    } in events.read()
    {
        for discord_channel_id in query.iter() {
            if !new_message.author.bot && &new_message.channel_id == &discord_channel_id.channel_id {
                match new_message
                    .content
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .as_slice()
                {
                    ["!pearl", "load", username] => {
                        chat_received_event.send(ChatReceivedEvent {
                            entity: discord_channel_id.entity,
                            packet: ChatPacket::System(Arc::new(ClientboundSystemChatPacket {
                                overlay: true,
                                content: format!("{} whispers: !pearl load", username).into(),
                            })),
                        });
                    }
                    _ => {
                        error!("Invalid Discord Command")
                    }
                }
            }
        }
    }
}
