///
/// Very simple ping-pong example.
/// Start the server with `cargo run --example ping_server`
/// Start the client with `cargo run --example ping_client`
///
/// The server will respond to a Ping event with a Pong event.
///
use bevy::prelude::*;
use bevy_client_server_events::{
    server::{
        ReceiveFromClient, ReceiveFromClientPlugin, SendToClient, SendToClientPlugin, StartServer,
    },
    ClientServerEventsPlugin,
};
use serde::{Deserialize, Serialize};

// Identical to ping_client.rs - this would typically be a shared library
#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Ping;

// Identical to ping_client.rs - this would typically be a shared library
#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Pong;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientServerEventsPlugin::default_server()) // One channel with ID of 0.
        .add_plugins(SendToClientPlugin::<0, Pong>::default()) // Use channel 0 for sending a Pong.
        .add_plugins(ReceiveFromClientPlugin::<0, Ping>::default()) // Use channel 0 for receiving a Ping.
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut start_server: EventWriter<StartServer>) {
    start_server.send(StartServer::default()); // Binds to 127.0.0.1:5000 by default.
}

fn update(
    mut ping: EventReader<ReceiveFromClient<Ping>>,
    mut pong: EventWriter<SendToClient<Pong>>,
) {
    for received in ping.iter() {
        println!("Client {} Sent: {:?}", received.client_id, received.content);
        pong.send(SendToClient {
            client_id: received.client_id,
            content: Pong,
        });
    }
}
