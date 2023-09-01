# Bevy Client Server Events

[![Bevy Client Server Events](https://github.com/edouardpoitras/bevy_client_server_events/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/edouardpoitras/bevy_client_server_events/actions/workflows/rust.yml)
[![Latest version](https://img.shields.io/crates/v/bevy_client_server_events.svg)](https://crates.io/crates/bevy_client_server_events)
[![Documentation](https://docs.rs/bevy_client_server_events/badge.svg)](https://docs.rs/bevy_client_server_events)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Simple event-based client-server networking library for Bevy.

Easily send bevy events to/from a client/server.

Builds off of the renet/bevy_renet library and attempts to simplify the configuration and management of types to be sent through a network.

**Goals**:
- Simplified network setup and configuration
- Easily send any types to/from a client/server

**Non-Goals**:
- Highly performant networking library
- Highly customizable networking library

**Todo**:
- Support secure authentication

## Plugins

This library comes with 6 different plugins.

#### ClientServerEventsPlugin

This plugin must be added to utilize any of the other plugins.

The endpoint type (`Client` or `Server`) and channels are specified in this plugin configuration.

Here's an example of a server with 3 available channels.

```rust,ignore
App::new()
    .add_plugins(ClientServerEventsPlugin::default_with_channels(
        EndpointType::Server, 0..3
    )); // Server with three channels (IDs: 0, 1, 2)
```

#### SendToServerPlugin

Add this client plugin (one or more times) to signal that a specific type will be sent to a server on a particular channel.

```rust,ignore
App::new()
    // ..
    .add_plugins(SendToServerPlugin::<0, CustomType1>::default() // CustomType1 will be sent to the server on channel 0.
    .add_plugins(SendToServerPlugin::<1, CustomType2>::default() // CustomType2 will be sent to the server on channel 1.
    // ..
```

Inside your systems, you can use events to send CustomType1 and CustomType2 to the server:

```rust, ignore
fn update(
    mut send_custom_type1: EventWriter<SendToServer<CustomType1>>,
    mut send_custom_type2: EventWriter<SendToServer<CustomType2>>,
) {
    send_custom_type1.send(SendToServer { content: CustomType1 { ... }});
    // ...
}
```

#### ReceiveFromServerPlugin

Add this client plugin (one or more times) to signal that a specific type will be received from the server on a particular channel.

```rust,ignore
App::new()
    // ..
    .add_plugins(ReceiveFromServerPlugin::<0, CustomType>::default() // CustomType will be sent to the server on channel 0.
    // ..
```

Inside your systems, you can use events to receive CustomType from the server:

```rust, ignore
fn update(
    mut receive_custom_type: EventReader<ReceiveFromServer<CustomType>>,
) {
    for ReceiveFromServer { content } in receive_custom_type.iter() {
        // content is our CustomType.
        // ...
    }
}
```

#### SendToClientPlugin and SendToClientsPlugin

Add this server plugin (one or more times) to signal that a specific type will be sent to the client on a particular channel.

```rust,ignore
App::new()
    // ..
    .add_plugins(SendToClientPlugin::<0, CustomType>::default() // CustomType will be sent to clients on channel 0.
    // ..
```

Inside your systems, you can use events to send CustomType to clients:

```rust, ignore
fn update(
    mut send_custom_type: EventWriter<SendToClient<CustomType>>,
) {
    let client_id = // ...
    send_custom_type.send(SendToClient { client_id, content: CustomType { ... }});
}
```

`SendToClientsPlugin` works the same way except that each message is boadcast to all clients:

```rust,ignore
App::new()
    // ..
    .add_plugins(SendToClientsPlugin::<1, CustomType>::default() // CustomType will be sent to all clients on channel 1.
    // ..

fn update(
    mut send_custom_type: EventWriter<SendToClients<CustomType>>,
) {
    send_custom_type.send(SendToClients { content: CustomType { ... }});
}
```

#### ReceiveFromClientPlugin

Add this server plugin (one or more times) to signal that a specific type will be received from clients on a particular channel.

```rust,ignore
App::new()
    // ..
    .add_plugins(ReceiveFromClientPlugin::<0, CustomType>::default() // CustomType will be received from clients on channel 0.
    // ..
```

Inside your systems, you can use events to receive CustomType from clients.

```rust, ignore
fn update(
    mut receive_custom_type: EventReader<ReceiveFromClient<CustomType>>,
) {
    for ReceiveFromClient { client_id, content } in receive_custom_type.iter() {
        // content is our CustomType.
        // ...
    }
}
```

## Channels

Channels (from renet) allows you to configure memory usage as well as the send type (`Unreliable`, `ReliableOrdered`, or `ReliableUnordered`). The default channel is `ReliableOrdered` with a resend time of 300ms.

Channels are configured when adding the `ClientServerEventsPlugin`. This configuration will determine the channel IDs available for the application. Then you can tie channels to certain types by adding the appropriate plugins:
- SendToServerPlugin::<`channel_id`, `MyType`>::default()
- ReceiveFromServerPlugin::<`channel_id`, `MyType`>::default()
- SendToClientPlugin::<`channel_id`, `MyType`>::default()
- SendToClientsPlugin::<`channel_id`, `MyType`>::default()
- ReceiveFromClientPlugin::<`channel_id`, `MyType`>::default()

See `examples/clients_example.rs` for different channel configuration options.

Each channel can only send and receive one type. While not recommended, the types can be distinct (ie: sending TypeA and receiving TypeB on channel 0 is fine), but you can not overload sending or receiving with more than one type (ie: sending TypeA and TypeB on channel 1 will cause deserialization issues).

The clients and server channel/type bindings must match (If client is sending CustomType on channel 0, the server must receive CustomType on channel 0 as well).

The simplest approach is to assign types to channels across your clients and server code regardless of sending or receiving (ie: TypeA == channel 0, TypeB == channel 1, TypeC == channel2, etc).

This was the intention for the design of this library (getting rid of channel IDs entirely), but I haven't yet figured out how to enforce that with the rust compiler.

## Examples

See the `examples/ping_server.rs` and `examples/ping_client.rs` files for a simple ping-pong example.

In one terminal session: `cargo run --example simple_server.rs`

In another terminal session: `cargo run --example simple_client.rs`

With the client window in focus, hit `ENTER` to send a Ping. The server will respond with a Pong.

### Overview

Assuming you have the following types in your project:

```rust,ignore
#[derive(Event, Serialize, Deserialize)]
pub struct MessageForServer;

#[derive(Event, Serialize, Deserialize)]
pub struct MessageForClient;
```

Here's a stripped down example of a client:

```rust,ignore
// ...
App::new()
    // ...
    .add_plugins(ClientServerEventsPlugin::default_client()) // Only one channel with ID of 0 by default.
    .add_plugins(SendToServerPlugin::<0, MessageForServer>::default()) // Use channel 0 for sending a MessageForServer.
    .add_plugins(ReceiveFromServerPlugin::<0, MessageForClient>::default()) // Use channel 0 for receiving a MessageForClient.
    // ...
// ...

fn connect(mut connect_to_server: EventWriter<ConnectToServer>) {
    connect_to_server.send(ConnectToServer::default()); // Connects to 127.0.0.1:5000 by default.
}

fn update(
    mut send_message: EventWriter<SendToServer<MessageForServer>>,
    mut message_received: EventReader<ReceiveFromServer<MessageForClient>>,
) {
    // ...
    send_message.send(SendToServer { content: MessageForServer });
    // ...
    for ReceiveFromServer { client_id, content } in message_received.iter() {
        // Do something with content (MessageForClient).
        // ...
    }
}
```

And a stripped down example for a server:

```rust,ignore
// ...
App::new()
    // ...
    .add_plugins(ClientServerEventsPlugin::default_server()) // Only one channel with ID of 0 by default.
    .add_plugins(ReceiveFromClientPlugin::<0, MessageForServer>::default()) // Use channel 0 for receiving a MessageForServer.
    .add_plugins(SendToClientPlugin::<0, MessageForClient>::default()) // Use channel 0 for sending a MessageForClient.
    // ...
// ...

fn setup(mut start_server: EventWriter<StartServer>) {
    start_server.send(StartServer::default()); // Binds to 127.0.0.1:5000 by default.
}

fn update(
    mut message_received: EventReader<ReceiveFromClient<MessageForServer>>,
    mut send_message: EventWriter<SendToClient<MessageForClient>>,
) {
    for ReceiveFromClient { client_id, content } in message_received.iter() {
        // Do something with content (MessageForServer).
        // ...
    }
    // ...
    send_message.send(SendToServer { content: MessageForClient });
    // ...
}

```

## Bevy Compatibility

|bevy|bevy_client_server_events|
|---|---|
|0.11|0.1|