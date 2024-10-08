///
/// Example using various features - IP/port hard-coded as default of 127.0.0.1:5000.
/// See the chat example for configurable IP/port.
/// Start the server with `cargo run --example features -- -s`
/// Start the client with `cargo run --example features
///
/// The server and client will use encryption to communicate.
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
    client::{ConnectToServer, DisconnectFromServer, ReceiveFromServer, SendToServer},
    client_server_events_plugin,
    server::{
        ClientConnected, ClientDisconnected, ReceiveFromClient, SendToClient, SendToClients,
        StartServer, StopServer,
    },
    string_to_key, NetcodeTransportError, NetworkConfig,
};
use renet::SendType;
use serde::{Deserialize, Serialize};
use std::{env, time::Duration};

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerMovement {
    pub x: f32,
    pub y: f32,
}

#[derive(Event, Serialize, Deserialize)]
pub struct ServerResponse {
    pub message: String,
}

#[derive(Event, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub message: String,
}

const SHARED_KEY: &str = "secret";

fn main() {
    let mut args = env::args();
    args.next(); // Don't care about the program name.
    let is_server: bool = args.next() == Some("-s".to_string());
    let mut app = App::new();
    client_server_events_plugin!(
        app,
        PlayerMovement => NetworkConfig::default(),
        ServerResponse => NetworkConfig::default(),
        BroadcastMessage => NetworkConfig {
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(1000),
            }
        }
    );
    if is_server {
        app.add_plugins(DefaultPlugins)
            .add_systems(Startup, setup_server)
            .add_systems(
                Update,
                (
                    update_server,
                    periodic_server_broadcast,
                    log_connections_on_server,
                    handle_errors,
                ),
            )
            .run();
    } else {
        app.add_plugins(DefaultPlugins)
            .add_systems(Startup, setup_client)
            .add_systems(Update, (update_client, handle_errors))
            .run();
    }
}

fn setup_server(mut start_server: EventWriter<StartServer>) {
    let key = string_to_key(SHARED_KEY);
    start_server.send(StartServer {
        private_key: Some(key),
        ..Default::default()
    });
}

fn setup_client(mut connect_to_server: EventWriter<ConnectToServer>) {
    let key = string_to_key(SHARED_KEY);
    connect_to_server.send(ConnectToServer {
        private_key: Some(key),
        ..Default::default()
    });
}

fn update_server(
    input: Res<ButtonInput<KeyCode>>,
    mut start_server_events: EventWriter<StartServer>,
    mut stop_server_events: EventWriter<StopServer>,
    mut player_movement_events: EventReader<ReceiveFromClient<PlayerMovement>>,
    mut server_response_events: EventWriter<SendToClient<ServerResponse>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        stop_server_events.send(StopServer);
        println!("Stopping server");
    } else if input.just_pressed(KeyCode::Enter) {
        let key = string_to_key(SHARED_KEY);
        start_server_events.send(StartServer {
            private_key: Some(key),
            ..Default::default()
        });
        println!("Starting server");
    }

    for ReceiveFromClient { client_id, content } in player_movement_events.read() {
        println!(
            "Player Movement Received from Client {}: {:?}",
            *client_id, content
        );
        server_response_events.send(SendToClient {
            client_id: *client_id,
            content: ServerResponse {
                message: "Player Movement Processed by Server".to_string(),
            },
        });
    }
}

fn periodic_server_broadcast(
    mut broadcast_events: EventWriter<SendToClients<BroadcastMessage>>,
    mut frames: Local<u64>,
) {
    *frames += 1;
    if *frames % 500 == 0 {
        broadcast_events.send(SendToClients {
            content: BroadcastMessage {
                message: format!("Broadcast: Server has been running for {} frames", *frames),
            },
        });
    }
}

fn log_connections_on_server(
    mut player_connected: EventReader<ClientConnected>,
    mut player_disconnected: EventReader<ClientDisconnected>,
) {
    for player_connected in player_connected.read() {
        println!("Player Connected: {:?}", player_connected);
    }
    for player_disconnected in player_disconnected.read() {
        println!("Player Disconnected: {:?}", player_disconnected);
    }
}

fn update_client(
    input: Res<ButtonInput<KeyCode>>,
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
    } else if input.just_pressed(KeyCode::Enter) {
        let key = string_to_key(SHARED_KEY);
        connect_events.send(ConnectToServer {
            private_key: Some(key),
            ..Default::default()
        });
        println!("Reconnecting to server");
    }

    for server_response in server_response_events.read() {
        println!("Server Response: {}", server_response.content.message);
    }

    for broadcast_message in broadcast_events.read() {
        println!("Broadcast: {}", broadcast_message.content.message);
    }
}

fn handle_errors(mut errors: EventReader<NetcodeTransportError>) {
    for error in errors.read() {
        println!("Networking Error: {:?}", error);
    }
}
