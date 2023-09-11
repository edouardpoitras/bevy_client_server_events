use bevy::prelude::{Commands, Event, EventReader, EventWriter, Res, ResMut};
use bevy_renet::renet::{transport::ClientAuthentication, ConnectionConfig, RenetClient};
use renet::transport::{NetcodeClientTransport, NETCODE_USER_DATA_BYTES};
use serde::de::DeserializeOwned;
use serde::Serialize;

use std::net::UdpSocket;
use std::time::SystemTime;

use crate::NetworkConfigs;

#[derive(Debug, Event)]
pub struct ConnectToServer {
    pub server_ip: String,
    pub server_port: u16,
    pub client_id: Option<u64>,
    pub protocol_id: u64,
    pub user_data: Option<[u8; NETCODE_USER_DATA_BYTES]>,
}

impl Default for ConnectToServer {
    fn default() -> Self {
        Self {
            server_ip: "127.0.0.1".to_string(),
            server_port: 5000,
            client_id: None,
            protocol_id: 1,
            user_data: None,
        }
    }
}

impl ConnectToServer {
    fn get_client_and_transport(
        &self,
        channel_configs: NetworkConfigs,
    ) -> (RenetClient, NetcodeClientTransport) {
        let client = RenetClient::new(ConnectionConfig {
            available_bytes_per_tick: channel_configs.available_bytes_per_tick,
            server_channels_config: channel_configs.clone().into(),
            client_channels_config: channel_configs.into(),
        });
        let server_addr = format!("{}:{}", self.server_ip, self.server_port)
            .parse()
            .unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = self.client_id.unwrap_or(current_time.as_millis() as u64);
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: self.protocol_id,
            server_addr,
            user_data: self.user_data,
        };
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
        (client, transport)
    }
}

#[derive(Debug, Event)]
pub struct DisconnectFromServer;

#[derive(Debug, Event)]
pub struct ReceiveFromServer<T: Event + Serialize + DeserializeOwned> {
    pub content: T,
}

#[derive(Debug, Event)]
pub struct SendToServer<T: Event + Serialize + DeserializeOwned> {
    pub content: T,
}

pub fn client_initiates_connection_to_server(
    mut connect_to_server_events: EventReader<ConnectToServer>,
    channel_configs: Res<NetworkConfigs>,
    mut commands: Commands,
) {
    for connect_to_server in connect_to_server_events.iter() {
        let (client, transport) =
            connect_to_server.get_client_and_transport(channel_configs.clone());
        commands.insert_resource(client);
        commands.insert_resource(transport);
    }
}

pub fn client_disconnects_from_server(
    mut disconnect_from_server_events: EventReader<DisconnectFromServer>,
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<NetcodeClientTransport>,
    mut commands: Commands,
) {
    for _ in disconnect_from_server_events.iter() {
        client.disconnect();
        transport.disconnect();
        commands.remove_resource::<RenetClient>();
        commands.remove_resource::<NetcodeClientTransport>();
    }
}

pub fn client_receives_messages_from_server<
    const I: u8,
    T: Event + Serialize + DeserializeOwned,
>(
    mut client: ResMut<RenetClient>,
    mut server_message_received_events: EventWriter<ReceiveFromServer<T>>,
) {
    while let Some(message) = client.receive_message(I) {
        let (server_message, _) =
            bincode::serde::decode_from_slice(&message, bincode::config::standard()).unwrap();
        server_message_received_events.send(ReceiveFromServer {
            content: server_message,
        });
    }
}

pub fn client_sends_messages_to_server<const I: u8, T: Event + Serialize + DeserializeOwned>(
    mut client: ResMut<RenetClient>,
    mut send_message_to_server_events: EventReader<SendToServer<T>>,
) {
    for message in send_message_to_server_events.iter() {
        let payload =
            bincode::serde::encode_to_vec(&message.content, bincode::config::standard()).unwrap();
        client.send_message(I, payload);
    }
}
