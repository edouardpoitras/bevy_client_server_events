///
/// Very simple ping-pong example - IP/port hard-coded as default of 127.0.0.1:5000.
/// See the chat example for configurable IP/port.
/// Start the server with `cargo run --example ping -- -s`
/// Start the client with `cargo run --example ping`
///
/// With the client window in focus, press `ENTER` to send a Ping.
/// The server will respond to a Ping event with a Pong event.
///
use bevy::prelude::*;
use bevy_client_server_events::{
    client::{ConnectToServer, ReceiveFromServer, SendToServer},
    client_server_events_plugin,
    server::{ReceiveFromClient, SendToClient, StartServer},
    NetworkConfig,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Ping;

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Pong;

fn main() {
    let mut args = env::args();
    args.next(); // Don't care about the program name.
    let is_server: bool = args.next() == Some("-s".to_string());
    let mut app = App::new();
    client_server_events_plugin!(
        app,
        Ping => NetworkConfig::default(),
        Pong => NetworkConfig::default()
    );
    if is_server {
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, setup_server)
            .add_systems(Update, update_server)
            .run();
    } else {
        app.add_plugins(DefaultPlugins)
            .add_systems(Startup, setup_client)
            .add_systems(Update, update_client)
            .run();
    }
}

fn setup_server(mut start_server: EventWriter<StartServer>) {
    start_server.send(StartServer::default()); // Binds to 127.0.0.1:5000 by default.
}

fn setup_client(mut connect_to_server: EventWriter<ConnectToServer>) {
    connect_to_server.send(ConnectToServer::default()); // Connects to 127.0.0.1:5000 by default.
}

fn update_server(
    mut ping: EventReader<ReceiveFromClient<Ping>>,
    mut pong: EventWriter<SendToClient<Pong>>,
) {
    for received in ping.read() {
        println!("Client {} Sent: {:?}", received.client_id, received.content);
        pong.send(SendToClient {
            client_id: received.client_id,
            content: Pong,
        });
    }
}

fn update_client(
    input: Res<ButtonInput<KeyCode>>,
    mut ping: EventWriter<SendToServer<Ping>>,
    mut pong: EventReader<ReceiveFromServer<Pong>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        ping.send(SendToServer { content: Ping });
    }
    for response in pong.read() {
        println!("Server Response: {:?}", response.content);
    }
}
