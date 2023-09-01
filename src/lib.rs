#![doc = include_str!("../README.md")]
use std::{ops::Range, time::Duration};

use bevy::prelude::{not, resource_exists, App, IntoSystemConfigs, Plugin, PostUpdate, Resource};
use bevy_renet::{
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use client::{
    client_disconnects_from_server, client_initiates_connection_to_server, ConnectToServer,
    DisconnectFromServer,
};
use renet::{RenetClient, RenetServer};
use server::{
    server_starts, server_stops, server_tracks_connected_and_disconnected_clients, ClientConnected,
    ClientDisconnected, StartServer, StopServer,
};

pub use renet::{transport::NetcodeTransportError, SendType};
pub mod client;
pub mod server;

pub struct ClientServerEventsPlugin {
    pub endpoint_type: EndpointType,
    pub channels_config: ChannelConfigs,
}

impl ClientServerEventsPlugin {
    pub fn default_client() -> Self {
        Self {
            endpoint_type: EndpointType::Client,
            channels_config: ChannelConfigs::default(),
        }
    }

    pub fn default_server() -> Self {
        Self {
            endpoint_type: EndpointType::Server,
            channels_config: ChannelConfigs::default(),
        }
    }

    pub fn default_with_channels(endpoint_type: EndpointType, channel_ids: Range<u8>) -> Self {
        Self {
            endpoint_type,
            channels_config: ChannelConfigs::default_with_channel_ids(channel_ids),
        }
    }
}

impl Plugin for ClientServerEventsPlugin {
    fn build(&self, app: &mut App) {
        match self.endpoint_type {
            EndpointType::Client => {
                app.insert_resource(self.channels_config.clone())
                    .add_plugins(RenetClientPlugin)
                    .add_plugins(NetcodeClientPlugin)
                    .add_event::<ConnectToServer>()
                    .add_event::<DisconnectFromServer>()
                    .add_systems(
                        PostUpdate,
                        client_initiates_connection_to_server
                            .run_if(not(resource_exists::<RenetClient>())),
                    )
                    .add_systems(
                        PostUpdate,
                        client_disconnects_from_server.run_if(resource_exists::<RenetClient>()),
                    );
            },
            EndpointType::Server => {
                app.insert_resource(self.channels_config.clone())
                    .add_plugins(RenetServerPlugin)
                    .add_plugins(NetcodeServerPlugin)
                    .add_event::<StartServer>()
                    .add_event::<StopServer>()
                    .add_event::<ClientConnected>()
                    .add_event::<ClientDisconnected>()
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
                    );
            },
        }
    }
}

pub enum EndpointType {
    Client,
    Server,
}

#[derive(Clone, Resource)]
pub struct ChannelConfigs {
    pub configs: Vec<ChannelConfig>,
    pub available_bytes_per_tick: u64,
}

impl Default for ChannelConfigs {
    fn default() -> Self {
        Self {
            configs: vec![ChannelConfig::default()],
            available_bytes_per_tick: 60_000,
        }
    }
}

impl ChannelConfigs {
    pub fn default_with_channel_ids(num_of_channels: Range<u8>) -> Self {
        let mut configs = Vec::new();
        for channel_id in num_of_channels {
            configs.push(ChannelConfig::default_with_channel_id(channel_id));
        }
        Self {
            configs,
            ..Default::default()
        }
    }
}

impl From<ChannelConfigs> for Vec<renet::ChannelConfig> {
    fn from(val: ChannelConfigs) -> Self {
        val.configs.into_iter().map(Into::into).collect()
    }
}

#[derive(Clone)]
pub struct ChannelConfig {
    pub channel_id: u8,
    pub max_memory_usage_bytes: usize,
    pub send_type: SendType,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            channel_id: 0,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(300),
            },
        }
    }
}

impl From<ChannelConfig> for renet::ChannelConfig {
    fn from(val: ChannelConfig) -> Self {
        renet::ChannelConfig {
            channel_id: val.channel_id,
            max_memory_usage_bytes: val.max_memory_usage_bytes,
            send_type: val.send_type,
        }
    }
}

impl ChannelConfig {
    pub fn default_with_channel_id(channel_id: u8) -> ChannelConfig {
        ChannelConfig {
            channel_id,
            ..Default::default()
        }
    }
}
