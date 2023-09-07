///
/// Chat client/server example.
/// Start the server with `cargo run --example chat_server`
/// Start the client with `cargo run --example chat_client`
///
use bevy::prelude::*;
use bevy_client_server_events::{
    server::{
        ClientConnected, ClientDisconnected, ReceiveFromClient, ReceiveFromClientPlugin,
        SendToClients, SendToClientsPlugin, StartServer,
    },
    ClientServerEventsPlugin,
};
use serde::{Deserialize, Serialize};
use std::env;

// Identical to chat_client.rs - this would typically be a shared library
#[derive(Event, Serialize, Deserialize)]
pub struct Message(String);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(ClientServerEventsPlugin::default_server())
        .add_plugins(SendToClientsPlugin::<0, Message>::default()) // Use channel 0 to send messages to clients.
        .add_plugins(ReceiveFromClientPlugin::<0, Message>::default()) // Use channel 0 to receive messages from clients.
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut start_server: EventWriter<StartServer>) {
    let mut args = env::args();
    args.next();
    let ip = if let Some(ip) = args.next() {
        ip
    } else {
        "127.0.0.1".to_string()
    };
    let port = if let Some(port) = args.next() {
        port.parse::<u16>().unwrap()
    } else {
        9000
    };
    println!("Starting chat server at {}:{}", ip.clone(), port);
    start_server.send(StartServer {
        ip,
        port,
        ..Default::default()
    });
}

fn update(
    mut client_messages: EventReader<ReceiveFromClient<Message>>,
    mut server_messages: EventWriter<SendToClients<Message>>,
    mut client_connected: EventReader<ClientConnected>,
    mut client_disconnected: EventReader<ClientDisconnected>,
) {
    for ReceiveFromClient {
        client_id,
        content: Message(message),
    } in client_messages.iter()
    {
        println!("{} sent: {}", client_id, message);
        server_messages.send(SendToClients {
            content: Message(format!("> {}: {}", client_id, message)),
        });
    }
    for ClientConnected { client_id } in client_connected.iter() {
        println!("{} has connected", client_id);
        server_messages.send(SendToClients {
            content: Message(format!("> {} has joined the chat!", client_id)),
        })
    }
    for ClientDisconnected {
        client_id,
        reason: _,
    } in client_disconnected.iter()
    {
        println!("{} has disconnected", client_id);
        server_messages.send(SendToClients {
            content: Message(format!("> {} has left the chat!", client_id)),
        })
    }
}
