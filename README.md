# ** DEPRECATED - no longer maintained **

This crate is now deprecated. Consider `bevy_replicon` or `lightyear`.

# Bevy Client Server Events

[![Bevy Client Server Events](https://github.com/edouardpoitras/bevy_client_server_events/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/edouardpoitras/bevy_client_server_events/actions/workflows/rust.yml)
[![Latest version](https://img.shields.io/crates/v/bevy_client_server_events.svg)](https://crates.io/crates/bevy_client_server_events)
[![Documentation](https://docs.rs/bevy_client_server_events/badge.svg)](https://docs.rs/bevy_client_server_events)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Simple event-based client-server networking library for Bevy.

Easily send bevy events to/from a client/server without worrying about serialization or network transport details.

Builds off of the renet/bevy_renet library and attempts to simplify the configuration and management of types to be sent through a network.

**Goals**:
- Simplified network setup and configuration
- Easily send any types to/from a client/server

**Requirements**:
- `serde` to derive `Serialize` and `Deserialize` on your types

## Events

The following events are useful for servers:
- `EventWriter<StartServer>` - Send this event to start a server
- `EventWriter<StopServer>` - Send this event to stop a running server
- `EventReader<ClientConnected>` - Received whenever a new client is connected
- `EventReader<ClientDisconnected>` - Received whenever a client has disconnected
- `EventReader<ReceiveFromClient<T>>` - Received whenever a client has sent type T to the server
- `EventWriter<SendToClient<T>>` - Send this event to have a particular client receive type T
- `EventWriter<SendToClients<T>>` - Send this event to have all connected clients receive type T

The following events are useful for clients:
- `EventWriter<ConnectToServer>` - Send this event to connect to a server
- `EventWriter<DisconnectFromServer>` - Send this event to disconnect from the server
- `EventWriter<SendToServer<T>>` - Send this event to have the server receive type T
- `EventReader<ReceiveFromServer<T>>` - Received whenever the server has sent type T to the client

Both the client and the server can receive the `EventReader<NetcodeTransportError>` events to deal with networking errors.

## Examples

There are a few examples in the `examples/` directory.

### Ping

See the `examples/ping.rs` file for a simple ping-pong example.

In one terminal session, start the server: `cargo run --example ping -- -s`

In another terminal session, connect with a client: `cargo run --example ping`

With the client window in focus, hit `ENTER` to send a Ping. The server will respond with a Pong.

In this example, we want to send a `Ping` event to the server and receive a `Pong` event in return.

```rust,ignore
#[derive(Event, Serialize, Deserialize)]
pub struct Ping;

#[derive(Event, Serialize, Deserialize)]
pub struct Pong;
```

When setting up our `App`, we need to pass it to the `client_server_events_plugin` macro and provide all the types to be sent over the network.

```rust,ignore
fn main() {
    // ...
    let mut app = App::new();
    client_server_events_plugin!(
        app,
        Ping => NetworkConfig::default(),
        Pong => NetworkConfig::default()
    );
    // ...
```

You can provide type-specific network configuration, such as reliability, resend time, max memory usage, etc.

The macro should be run regardless of whether this instance will be a server or a client.

You can choose to start a server instance or connect to a server as a client using events.

```rust,ignore
fn start_server(mut start_server: EventWriter<StartServer>) {
    start_server.send(StartServer::default()); // Binds to 127.0.0.1:5000 with no encryption by default.
}

fn connect_as_client(mut connect_to_server: EventWriter<ConnectToServer>) {
    connect_to_server.send(ConnectToServer::default()); // Connects to 127.0.0.1:5000 with no encryption by default.
}
```

Then you can send/receive events as desired.

```rust,ignore
fn update_client(
    mut send_ping: EventWriter<SendToServer<Ping>>,
    mut receive_pong: EventReader<ReceiveFromServer<Pong>>,
) {
    // ...
    send_ping.send(SendToServer { content: Ping });
    // ...
    for ReceiveFromServer { content } in receive_pong.read() {
        // Do something with content (Pong).
        // ...
    }
}

fn update_server(
    mut receive_ping: EventReader<ReceiveFromClient<Ping>>,
    mut send_pong: EventWriter<SendToClient<Pong>>,
) {
    for ReceiveFromClient { client_id, content } in receive_ping.read() {
        // Do something with content (Ping).
        send_pong.send(SendToClient {
            client_id,
            content: Pong,
        });
    }
}
```

### Features Example

See the `examples/features.rs` file for examples of more features, such as encryption, broadcasting, networking error handling, and client connect/disconnect events.

In one terminal session, start the server: `cargo run --example features -- -s`

In another terminal session, connect with a client: `cargo run --example features`

The server and client will use encryption to communicate.

Every 500 frames the server will broadcast a message of it's frame count.

With focus on the server window:
- Hit `ESC` to stop the server
- Hit `ENTER` to start the server

With focus on the client window:
- Hit `ESC` to disconnect from the server
- Hit `ENTER` to reconnect to the server
- Hit `SPACE` to send a message of type `PlayerMovement`

The server will respond to the `PlayerMovement` message with a `ServerResponse` message.

## Other Networking Crates

This crate was created because I wanted the quickest and easiest way to send types through a network.

Other solutions seem to have me bogged down in transport/networking/channel details.

This crate is ideal for prototyping and small client-server games.

An alternative with more bells & whistles would be [bevy_replicon](https://github.com/lifescapegame/bevy_replicon).

A mature alternative with more customizability would be [bevy_renet](https://github.com/lucaspoffo/renet/tree/master/bevy_renet).

## Bevy Compatibility

|bevy|bevy_client_server_events|
|---|---|
|0.14|0.7|
|0.12|0.6|
|0.11|0.5|