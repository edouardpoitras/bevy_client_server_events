use bevy::prelude::{Commands, Event, EventReader, EventWriter, Res, ResMut};
use bevy_renet::renet::{
    transport::{ServerAuthentication, ServerConfig},
    ConnectionConfig, RenetServer,
};
use renet::{transport::NetcodeServerTransport, DisconnectReason, ServerEvent};
use serde::{de::DeserializeOwned, Serialize};

use std::net::UdpSocket;
use std::time::SystemTime;

use crate::NetworkConfigs;

#[derive(Debug, Event)]
pub struct StartServer {
    pub ip: String,
    pub port: u16,
    pub max_clients: usize,
    pub protocol_id: u64,
    pub available_bytes_per_tick: u64,
    pub private_key: Option<[u8; 32]>,
}

impl Default for StartServer {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 5000,
            max_clients: 64,
            protocol_id: 1,
            available_bytes_per_tick: 60_000,
            private_key: None,
        }
    }
}

impl StartServer {
    fn get_server_and_transport(
        &self,
        channel_configs: NetworkConfigs,
    ) -> (RenetServer, NetcodeServerTransport) {
        let server = RenetServer::new(ConnectionConfig {
            available_bytes_per_tick: self.available_bytes_per_tick,
            server_channels_config: channel_configs.clone().into(),
            client_channels_config: channel_configs.into(),
        });
        let public_addr = format!("{}:{}", self.ip, self.port).parse().unwrap();
        let socket = UdpSocket::bind(public_addr).unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let authentication = if let Some(private_key) = self.private_key {
            ServerAuthentication::Secure { private_key }
        } else {
            ServerAuthentication::Unsecure
        };
        let server_config = ServerConfig {
            max_clients: self.max_clients,
            protocol_id: self.protocol_id,
            public_addr,
            authentication,
        };
        let transport = NetcodeServerTransport::new(current_time, server_config, socket).unwrap();
        (server, transport)
    }
}

#[derive(Debug, Event)]
pub struct StopServer;

#[derive(Debug, Event)]
pub struct ClientConnected {
    pub client_id: u64,
}

#[derive(Debug, Event)]
pub struct ClientDisconnected {
    pub client_id: u64,
    pub reason: DisconnectReason,
}

#[derive(Debug, Event)]
pub struct ReceiveFromClient<T: Event + Serialize + DeserializeOwned> {
    pub client_id: u64,
    pub content: T,
}

#[derive(Debug, Event)]
pub struct SendToClient<T: Event + Serialize + DeserializeOwned> {
    pub client_id: u64,
    pub content: T,
}

#[derive(Debug, Event)]
pub struct SendToClients<T: Event + Serialize + DeserializeOwned> {
    pub content: T,
}

pub fn server_starts(
    mut start_server_events: EventReader<StartServer>,
    channel_configs: Res<NetworkConfigs>,
    mut commands: Commands,
) {
    for start_server in start_server_events.iter() {
        let (server, transport) = start_server.get_server_and_transport(channel_configs.clone());
        commands.insert_resource(server);
        commands.insert_resource(transport);
    }
}

pub fn server_stops(
    mut stop_server_events: EventReader<StopServer>,
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<NetcodeServerTransport>,
    mut commands: Commands,
) {
    for _ in stop_server_events.iter() {
        server.disconnect_all();
        transport.disconnect_all(&mut server);
        commands.remove_resource::<RenetServer>();
        // bevy_renet crashes due to missing resource if we remove the transport on this tick.
        // Removing it on the next tick instead (see cleanup_transport).
        //commands.remove_resource::<NetcodeServerTransport>();
    }
}

pub fn server_tracks_connected_and_disconnected_clients(
    mut server_events: EventReader<ServerEvent>,
    mut client_connected_events: EventWriter<ClientConnected>,
    mut client_disconnected_events: EventWriter<ClientDisconnected>,
) {
    for server_event in server_events.iter() {
        match server_event {
            ServerEvent::ClientConnected { client_id } => {
                client_connected_events.send(ClientConnected {
                    client_id: *client_id,
                });
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                client_disconnected_events.send(ClientDisconnected {
                    client_id: *client_id,
                    reason: *reason,
                });
            },
        }
    }
}

pub fn server_receives_messages_from_clients<
    const I: u8,
    T: Event + Serialize + DeserializeOwned,
>(
    mut server: ResMut<RenetServer>,
    mut client_message_events: EventWriter<ReceiveFromClient<T>>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, I) {
            let (content, _): (T, usize) =
                bincode::serde::decode_from_slice(&message, bincode::config::standard()).unwrap();
            client_message_events.send(ReceiveFromClient { client_id, content });
        }
    }
}

pub fn server_sends_messages_to_clients<const I: u8, T: Event + Serialize + DeserializeOwned>(
    mut server: ResMut<RenetServer>,
    mut send_message_to_client_events: EventReader<SendToClient<T>>,
) {
    for message in send_message_to_client_events.iter() {
        let payload =
            bincode::serde::encode_to_vec(&message.content, bincode::config::standard()).unwrap();
        server.send_message(message.client_id, I, payload);
    }
}

pub fn server_broadcasts_messages_to_clients<
    const I: u8,
    T: Event + Serialize + DeserializeOwned,
>(
    mut server: ResMut<RenetServer>,
    mut broadcast_message_events: EventReader<SendToClients<T>>,
) {
    for message in broadcast_message_events.iter() {
        let payload =
            bincode::serde::encode_to_vec(&message.content, bincode::config::standard()).unwrap();
        server.broadcast_message(I, payload);
    }
}

pub fn cleanup_transport(mut commands: Commands) {
    commands.remove_resource::<renet::transport::NetcodeServerTransport>();
}
