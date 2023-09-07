///
/// Chat client/server example.
/// Start the server with `cargo run --example chat_server`
/// Start the client with `cargo run --example chat_client`
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
use std::env;

// Identical to chat_server.rs - this would typically be a shared library
#[derive(Event, Serialize, Deserialize)]
pub struct Message(String);

#[derive(Component)]
struct ChatArea;

#[derive(Component)]
struct TextInput;

const MAX_LINES: usize = 10;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientServerEventsPlugin::default_client())
        .add_plugins(SendToServerPlugin::<0, Message>::default()) // Use channel 0 to send messages to the server.
        .add_plugins(ReceiveFromServerPlugin::<0, Message>::default()) // Use channel 0 to receive ServerResponse.
        .add_systems(Startup, setup)
        .add_systems(Update, (receive_messages, handle_input))
        .run();
}

fn setup(mut commands: Commands, mut connect_to_server: EventWriter<ConnectToServer>) {
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
    println!("Connecting to {}:{}", ip, port);
    connect_to_server.send(ConnectToServer {
        server_ip: ip,
        server_port: port,
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
                    color: Color::rgb(0.8, 0.8, 0.8),
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

fn receive_messages(
    mut server_messages: EventReader<ReceiveFromServer<Message>>,
    mut chat_area: Query<&mut Text, (With<ChatArea>, Without<TextInput>)>,
) {
    for ReceiveFromServer {
        content: Message(message),
    } in server_messages.iter()
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
    mut characters: EventReader<ReceivedCharacter>,
    keyboard: Res<Input<KeyCode>>,
    mut client_messages: EventWriter<SendToServer<Message>>,
    mut text_input: Query<&mut Text, (With<TextInput>, Without<ChatArea>)>,
) {
    let mut text = text_input.single_mut();
    if keyboard.just_pressed(KeyCode::Return) {
        let message = text.sections[1].value.clone();
        text.sections[1].value.clear();
        client_messages.send(SendToServer {
            content: Message(message),
        });
    }
    if keyboard.just_pressed(KeyCode::Back) {
        text.sections[1].value.pop();
    }
    for ReceivedCharacter { char, window: _ } in characters.iter() {
        if !char.is_control() {
            text.sections[1].value.push(*char);
        }
    }
}
