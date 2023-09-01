///
/// Very simple ping-pong example.
/// Start the server with `cargo run --example ping_server`
/// Start the client with `cargo run --example ping_client`
///
/// With the client window in focus, press `ENTER` to send a Ping.
/// The server will respond to a Ping event with a Pong event.
///
use bevy::prelude::*;
use bevy_client_server_events::{
    client::{
        ConnectToServer, ReceiveFromServer, ReceiveFromServerPlugin, SendToServer,
        SendToServerPlugin,
    },
    ClientServerEventsPlugin,
};
use serde::{Deserialize, Serialize};

// Identical to ping_server.rs - this would typically be a shared library
#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Ping;

// Identical to ping_server.rs - this would typically be a shared library
#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Pong;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientServerEventsPlugin::default_client()) // One channel with ID of 0.
        .add_plugins(SendToServerPlugin::<0, Ping>::default()) // Use channel 0 for sending a Ping.
        .add_plugins(ReceiveFromServerPlugin::<0, Pong>::default()) // Use channel 0 for receiving a Pong.
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut connect_to_server: EventWriter<ConnectToServer>) {
    connect_to_server.send(ConnectToServer::default()); // Connects to 127.0.0.1:5000 by default.
}

fn update(
    input: Res<Input<KeyCode>>,
    mut ping: EventWriter<SendToServer<Ping>>,
    mut pong: EventReader<ReceiveFromServer<Pong>>,
) {
    if input.just_pressed(KeyCode::Return) {
        ping.send(SendToServer { content: Ping });
    }
    for response in pong.iter() {
        println!("Server Response: {:?}", response.content);
    }
}
