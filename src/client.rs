use bevy::prelude::{Commands, Event, EventReader, EventWriter, Res, ResMut};
use bevy_renet::renet::{transport::ClientAuthentication, ConnectionConfig, RenetClient};
use bincode::{Decode, Encode};
use renet::transport::{ConnectToken, NetcodeClientTransport, NETCODE_USER_DATA_BYTES};

use std::net::UdpSocket;
use std::time::SystemTime;

use crate::NetworkConfigs;

#[derive(Debug, Event)]
pub struct ConnectToServer {
    pub server_ip: String,
    pub server_port: u16,
    pub protocol_id: u64,
    pub available_bytes_per_tick: u64,
    pub client_id: Option<u64>,
    pub user_data: Option<[u8; NETCODE_USER_DATA_BYTES]>,
    pub expire_seconds: Option<u64>,
    pub timeout_seconds: Option<i32>,
    pub private_key: Option<[u8; 32]>,
}

impl Default for ConnectToServer {
    fn default() -> Self {
        Self {
            server_ip: "127.0.0.1".to_string(),
            server_port: 5000,
            protocol_id: 1,
            available_bytes_per_tick: 60_000,
            client_id: None,
            user_data: None,
            expire_seconds: None,
            timeout_seconds: None,
            private_key: None,
        }
    }
}

impl ConnectToServer {
    fn get_client_and_transport(
        &self,
        channel_configs: NetworkConfigs,
    ) -> (RenetClient, NetcodeClientTransport) {
        let client = RenetClient::new(ConnectionConfig {
            available_bytes_per_tick: self.available_bytes_per_tick,
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
        let authentication = if let Some(private_key) = self.private_key {
            let ud;
            let user_data = if self.user_data.is_some() {
                ud = self.user_data.unwrap();
                Some(&ud)
            } else {
                None
            };
            ClientAuthentication::Secure {
                connect_token: ConnectToken::generate(
                    current_time,
                    self.protocol_id,
                    self.expire_seconds.unwrap_or(86_400), // 1 day by default
                    client_id,
                    self.timeout_seconds.unwrap_or(-1), // No timeout by default
                    vec![server_addr],
                    user_data,
                    &private_key,
                )
                .unwrap(),
            }
        } else {
            ClientAuthentication::Unsecure {
                client_id,
                protocol_id: self.protocol_id,
                server_addr,
                user_data: self.user_data,
            }
        };
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
        (client, transport)
    }
}

#[derive(Debug, Event)]
pub struct DisconnectFromServer;

#[derive(Debug, Event)]
pub struct ReceiveFromServer<T: Event + Encode + Decode> {
    pub content: T,
}

#[derive(Debug, Event)]
pub struct SendToServer<T: Event + Encode + Decode> {
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

pub fn client_receives_messages_from_server<const I: u8, T: Event + Encode + Decode>(
    mut client: ResMut<RenetClient>,
    mut server_message_received_events: EventWriter<ReceiveFromServer<T>>,
) {
    while let Some(message) = client.receive_message(I) {
        let (server_message, _) =
            bincode::decode_from_slice(&message, bincode::config::standard()).unwrap();
        server_message_received_events.send(ReceiveFromServer {
            content: server_message,
        });
    }
}

pub fn client_sends_messages_to_server<const I: u8, T: Event + Encode + Decode>(
    mut client: ResMut<RenetClient>,
    mut send_message_to_server_events: EventReader<SendToServer<T>>,
) {
    for message in send_message_to_server_events.iter() {
        let payload =
            bincode::encode_to_vec(&message.content, bincode::config::standard()).unwrap();
        client.send_message(I, payload);
    }
}
