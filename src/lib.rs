#![doc = include_str!("../README.md")]
use std::time::Duration;

use renet::{RenetClient, RenetServer};

use bevy::prelude::{not, resource_exists, App, IntoSystemConfigs, Plugin, PostUpdate, Resource};

use bevy_renet::{
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};

use client::{
    client_disconnects_from_server, client_initiates_connection_to_server, ConnectToServer,
    DisconnectFromServer,
};

use server::{
    server_starts, server_stops, server_tracks_connected_and_disconnected_clients, ClientConnected,
    ClientDisconnected, StartServer, StopServer,
};

pub use bincode::{Decode, Encode};
pub use renet::{
    transport::NetcodeTransportError, RenetClient as Client, RenetServer as Server, SendType,
};
pub mod client;
pub mod macros;
pub mod server;

pub struct ClientServerEventsPlugin {
    pub channels_config: NetworkConfigs,
}

impl Plugin for ClientServerEventsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.channels_config.clone())
            .add_plugins(RenetServerPlugin)
            .add_plugins(NetcodeServerPlugin)
            .add_plugins(RenetClientPlugin)
            .add_plugins(NetcodeClientPlugin)
            .add_event::<StartServer>()
            .add_event::<StopServer>()
            .add_event::<ClientConnected>()
            .add_event::<ClientDisconnected>()
            .add_event::<ConnectToServer>()
            .add_event::<DisconnectFromServer>()
            .add_systems(
                PostUpdate,
                server_starts.run_if(not(resource_exists::<RenetServer>())),
            )
            .add_systems(
                PostUpdate,
                server_stops.run_if(resource_exists::<RenetServer>()),
            )
            .add_systems(
                PostUpdate,
                server_tracks_connected_and_disconnected_clients
                    .run_if(resource_exists::<RenetServer>()),
            )
            .add_systems(
                PostUpdate,
                client_initiates_connection_to_server.run_if(not(resource_exists::<RenetClient>())),
            )
            .add_systems(
                PostUpdate,
                client_disconnects_from_server.run_if(resource_exists::<RenetClient>()),
            );
    }
}

#[derive(Clone, Resource)]
pub struct NetworkConfigs(pub Vec<NetworkConfig>);

impl Default for NetworkConfigs {
    fn default() -> Self {
        Self(vec![NetworkConfig::default()])
    }
}

impl From<NetworkConfigs> for Vec<renet::ChannelConfig> {
    fn from(val: NetworkConfigs) -> Self {
        let mut renet_configs = Vec::new();
        for i in 0..val.0.len().min(u8::MAX as usize) {
            renet_configs.push(renet::ChannelConfig {
                channel_id: i as u8,
                max_memory_usage_bytes: val.0[i].max_memory_usage_bytes,
                send_type: val.0[i].send_type.clone(),
            });
        }
        renet_configs
    }
}

#[derive(Clone)]
pub struct NetworkConfig {
    pub send_type: SendType,
    pub max_memory_usage_bytes: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(300),
            },
        }
    }
}
