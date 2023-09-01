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
    server::{
        ClientConnected, ClientDisconnected, ReceiveFromClient, ReceiveFromClientPlugin,
        SendToClient, SendToClientPlugin, SendToClients, SendToClientsPlugin, StartServer,
        StopServer,
    },
    ClientServerEventsPlugin, EndpointType, NetcodeTransportError,
};
use serde::{Deserialize, Serialize};

// Identical to client_example.rs - this would typically be a shared library
#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerMovement {
    pub x: f32,
    pub y: f32,
}

// Identical to client_example.rs - this would typically be a shared library
#[derive(Event, Serialize, Deserialize)]
pub struct ServerResponse {
    pub message: String,
}

// Identical to client_example.rs - this would typically be a shared library
#[derive(Event, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub message: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientServerEventsPlugin::default_with_channels(
            EndpointType::Server,
            0..2, // Will create two default channels with IDs 0 and 1.
        ))
        .add_plugins(SendToClientPlugin::<0, ServerResponse>::default()) // Use channel 0 to send ServerResponse.
        .add_plugins(SendToClientsPlugin::<1, BroadcastMessage>::default()) // Use channel 1 to send BroadcastMessage.
        .add_plugins(ReceiveFromClientPlugin::<0, PlayerMovement>::default()) // Use channel 0 to receive PlayerMovement.
        .add_systems(Startup, setup)
        .add_systems(Update, (update, broadcast, log_connections, handle_errors))
        .run();
}

fn setup(mut start_server: EventWriter<StartServer>) {
    start_server.send(StartServer::default());
}

fn log_connections(
    mut player_connected: EventReader<ClientConnected>,
    mut player_disconnected: EventReader<ClientDisconnected>,
) {
    for player_connected in player_connected.iter() {
        println!("Player Connected: {:?}", player_connected);
    }
    for player_disconnected in player_disconnected.iter() {
        println!("Player Disconnected: {:?}", player_disconnected);
    }
}

fn update(
    input: Res<Input<KeyCode>>,
    mut start_server_events: EventWriter<StartServer>,
    mut stop_server_events: EventWriter<StopServer>,
    mut player_movement_events: EventReader<ReceiveFromClient<PlayerMovement>>,
    mut server_response_events: EventWriter<SendToClient<ServerResponse>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        stop_server_events.send(StopServer);
        println!("Stopping server");
    } else if input.just_pressed(KeyCode::Return) {
        start_server_events.send(StartServer::default());
        println!("Starting server");
    }

    for ReceiveFromClient { client_id, content } in player_movement_events.iter() {
        println!(
            "Player Movement Received from Client {}: {:?}",
            *client_id, content
        );
        server_response_events.send(SendToClient {
            client_id: *client_id,
            content: ServerResponse {
                message: "Player Movement Processed by Server".to_string(),
            },
        })
    }
}

fn broadcast(
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

fn handle_errors(mut errors: EventReader<NetcodeTransportError>) {
    for error in errors.iter() {
        println!("Networking Error: {:?}", error);
    }
}
