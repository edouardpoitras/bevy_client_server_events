///
/// Chat client/server example.
/// Start the server with `cargo run --example chat -- -s 127.0.0.1 5000`
/// Start the clients with `cargo run --example chat -- -c 127.0.0.1 5000`
///
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_client_server_events::{
    client::{ConnectToServer, ReceiveFromServer, SendToServer},
    client_server_events_plugin,
    server::{ClientConnected, ClientDisconnected, ReceiveFromClient, SendToClients, StartServer},
    NetworkConfig,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Resource)]
struct IpPort {
    ip: String,
    port: u16,
}

#[derive(Event, Serialize, Deserialize)]
pub struct Message(String);

#[derive(Component)]
struct ChatArea;

#[derive(Component)]
struct TextInput;

const MAX_LINES: usize = 10;

fn main() {
    let (is_server, ip, port) = parse_args();
    let mut app = App::new();
    client_server_events_plugin!(
        app,
        Message => NetworkConfig::default()
    );
    app.insert_resource(IpPort { ip, port });
    if is_server {
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, setup_server)
            .add_systems(Update, update_server)
            .run();
    } else {
        app.add_plugins(DefaultPlugins)
            .add_systems(Startup, setup_client)
            .add_systems(Update, (receive_messages, handle_input))
            .run();
    }
}

fn setup_server(ip_port: Res<IpPort>, mut start_server: EventWriter<StartServer>) {
    println!(
        "Starting chat server at {}:{}",
        ip_port.ip.clone(),
        ip_port.port
    );
    start_server.send(StartServer {
        ip: ip_port.ip.clone(),
        port: ip_port.port,
        ..Default::default()
    });
}

fn setup_client(
    ip_port: Res<IpPort>,
    mut commands: Commands,
    mut connect_to_server: EventWriter<ConnectToServer>,
) {
    println!("Client connecting to {}:{}", ip_port.ip, ip_port.port);
    connect_to_server.send(ConnectToServer {
        server_ip: ip_port.ip.clone(),
        server_port: ip_port.port,
        ..Default::default()
    });

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Text Input: ",
                TextStyle {
                    font_size: 24.0,
                    ..Default::default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..Default::default()
                },
            ),
        ]),
        TextInput,
    ));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 24.0,
                ..Default::default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            ..Default::default()
        }),
        ChatArea,
    ));
}

fn update_server(
    mut client_messages: EventReader<ReceiveFromClient<Message>>,
    mut server_messages: EventWriter<SendToClients<Message>>,
    mut client_connected: EventReader<ClientConnected>,
    mut client_disconnected: EventReader<ClientDisconnected>,
) {
    for ReceiveFromClient {
        client_id,
        content: Message(message),
    } in client_messages.read()
    {
        println!("{} sent: {}", client_id, message);
        server_messages.send(SendToClients {
            content: Message(format!("> {}: {}", client_id, message)),
        });
    }
    for ClientConnected { client_id } in client_connected.read() {
        println!("{} has connected", client_id);
        server_messages.send(SendToClients {
            content: Message(format!("> {} has joined the chat!", client_id)),
        });
    }
    for ClientDisconnected {
        client_id,
        reason: _,
    } in client_disconnected.read()
    {
        println!("{} has disconnected", client_id);
        server_messages.send(SendToClients {
            content: Message(format!("> {} has left the chat!", client_id)),
        });
    }
}

fn receive_messages(
    mut server_messages: EventReader<ReceiveFromServer<Message>>,
    mut chat_area: Query<&mut Text, (With<ChatArea>, Without<TextInput>)>,
) {
    for ReceiveFromServer {
        content: Message(message),
    } in server_messages.read()
    {
        let current_text = &chat_area.single_mut().sections[0].value.clone();
        let mut lines: Vec<&str> = current_text.split('\n').collect();
        lines.push(message);
        if lines.len() > MAX_LINES {
            lines.remove(0);
        }
        chat_area.single_mut().sections[0].value.clear();
        chat_area.single_mut().sections[0].value = lines.join("\n");
    }
}

fn handle_input(
    mut characters: EventReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client_messages: EventWriter<SendToServer<Message>>,
    mut text_input: Query<&mut Text, (With<TextInput>, Without<ChatArea>)>,
) {
    let mut text = text_input.single_mut();
    if keyboard.just_pressed(KeyCode::Enter) {
        let message = text.sections[1].value.clone();
        text.sections[1].value.clear();
        client_messages.send(SendToServer {
            content: Message(message),
        });
    }
    if keyboard.just_pressed(KeyCode::Backspace) {
        text.sections[1].value.pop();
    }
    for KeyboardInput {
        key_code: _,
        logical_key,
        state,
        window: _,
    } in characters.read()
    {
        // Only check for characters when the key is pressed.
        if !state.is_pressed() {
            continue;
        }

        // Note that some keys such as `Space` and `Tab` won't be detected as a character.
        // Instead, check for them as separate enum variants.
        match &logical_key {
            Key::Character(character) => {
                text.sections[1]
                    .value
                    .push(character.chars().last().unwrap());
            },
            Key::Space => {
                text.sections[1].value.push(' ');
            },
            _ => {},
        }
    }
}

fn parse_args() -> (bool, String, u16) {
    let mut args = env::args();
    args.next(); // Don't care about the program name.
    let is_server: bool = args.next() == Some("-s".to_string());
    let ip = if let Some(ip) = args.next() {
        ip
    } else {
        "127.0.0.1".to_string()
    };
    let port = if let Some(port) = args.next() {
        port.parse::<u16>().unwrap()
    } else {
        5000
    };
    (is_server, ip, port)
}
