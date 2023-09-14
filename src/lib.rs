#![doc = include_str!("../README.md")]
use std::time::Duration;

use renet::{RenetClient, RenetServer};

use bevy::prelude::{
    not, resource_exists, resource_removed, App, IntoSystemConfigs, Plugin, PostUpdate, PreUpdate,
    Resource,
};

use bevy_renet::{
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};

use client::{
    client_disconnects_from_server, client_initiates_connection_to_server, ConnectToServer,
    DisconnectFromServer,
};

use server::{
    cleanup_transport, server_starts, server_stops,
    server_tracks_connected_and_disconnected_clients, ClientConnected, ClientDisconnected,
    StartServer, StopServer,
};

pub use bincode::{Decode, Encode};
pub use renet::{
    transport::NetcodeTransportError, RenetClient as Client, RenetServer as Server, SendType,
};
pub mod client;
pub mod macros;
pub mod server;

///
/// Converts a string to a key that can be used for Authenticated connections.
/// Key is 32 bytes long, truncating and padding occurs otherwise.
///
pub fn string_to_key<K: Into<String>>(string: K) -> [u8; 32] {
    let mut key: [u8; 32] = [0; 32];
    string
        .into()
        .bytes()
        .zip(key.iter_mut())
        .for_each(|(b, ptr)| *ptr = b);
    key
}

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
                PreUpdate,
                cleanup_transport.run_if(resource_removed::<renet::RenetServer>()),
            )
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
