///
/// Example using various features.
/// Start the server with `cargo run --example server_example`
/// Start the client with `cargo run --example client_example`
///
/// Every 500 frames the server will broadcast a message of it's frame count.
///
/// With focus on the server window:
///     Hit 'ESC' to stop the server
///     Hit 'ENTER' to start the server
///
/// With focus on the client window:
///     Hit 'ESC' to disconnect from the server
///     Hit 'ENTER' to reconnect to the server
///     Hit 'SPACE' to send a message of type PlayerMovement
///
/// The server will respond to the PlayerMovement message with a ServerResponse message.
///
use bevy::prelude::*;
use bevy_client_server_events::{
    client::{
        ConnectToServer, DisconnectFromServer, ReceiveFromServer, ReceiveFromServerPlugin,
        SendToServer, SendToServerPlugin,
    },
    ChannelConfig, ChannelConfigs, ClientServerEventsPlugin, EndpointType, NetcodeTransportError,
    SendType,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Identical to server_example.rs - this would typically be a shared library
#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerMovement {
    pub x: f32,
    pub y: f32,
}

// Identical to server_example.rs - this would typically be a shared library
#[derive(Event, Serialize, Deserialize)]
pub struct ServerResponse {
    pub message: String,
}

// Identical to server_example.rs - this would typically be a shared library
#[derive(Event, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub message: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientServerEventsPlugin {
            endpoint_type: EndpointType::Client,
            channels_config: ChannelConfigs {
                configs: vec![
                    // Setup 2 channels with IDs 0 and 1.
                    ChannelConfig::default(), // channel_id = 0
                    ChannelConfig {
                        channel_id: 1,
                        max_memory_usage_bytes: 5 * 1024 * 1024,
                        send_type: SendType::ReliableOrdered {
                            resend_time: Duration::from_millis(300),
                        },
                    },
                ],
                available_bytes_per_tick: 60_000,
            },
        })
        .add_plugins(SendToServerPlugin::<0, PlayerMovement>::default()) // Use channel 0 to send PlayerMovement.
        .add_plugins(ReceiveFromServerPlugin::<0, ServerResponse>::default()) // Use channel 0 to receive ServerResponse.
        .add_plugins(ReceiveFromServerPlugin::<1, BroadcastMessage>::default()) // Use channel 1 to receive BroadcastMessage.
        .add_systems(Startup, setup)
        .add_systems(Update, (update, handle_errors))
        .run();
}

fn setup(mut connect_to_server: EventWriter<ConnectToServer>) {
    connect_to_server.send(ConnectToServer::default());
}

fn update(
    input: Res<Input<KeyCode>>,
    mut player_movement_events: EventWriter<SendToServer<PlayerMovement>>,
    mut disconnect_events: EventWriter<DisconnectFromServer>,
    mut connect_events: EventWriter<ConnectToServer>,
    mut server_response_events: EventReader<ReceiveFromServer<ServerResponse>>,
    mut broadcast_events: EventReader<ReceiveFromServer<BroadcastMessage>>,
) {
    if input.just_pressed(KeyCode::Space) {
        player_movement_events.send(SendToServer {
            content: PlayerMovement { x: 1.0, y: 1.0 },
        });
        println!("Sending Player Movement to Server");
    } else if input.just_pressed(KeyCode::Escape) {
        disconnect_events.send(DisconnectFromServer);
        println!("Disconnecting from server");
    } else if input.just_pressed(KeyCode::Return) {
        connect_events.send(ConnectToServer::default());
        println!("Reconnecting to server");
    }

    for server_response in server_response_events.iter() {
        println!("Server Response: {}", server_response.content.message);
    }

    for broadcast_message in broadcast_events.iter() {
        println!("Broadcast: {}", broadcast_message.content.message);
    }
}

fn handle_errors(mut errors: EventReader<NetcodeTransportError>) {
    for error in errors.iter() {
        println!("Networking Error: {:?}", error);
    }
}
