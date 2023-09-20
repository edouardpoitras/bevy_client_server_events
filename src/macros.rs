#[macro_export]
macro_rules! client_server_events_plugin {
    // TODO: Collapse into the next case and use a proper empty base-case.
    // In case a single type + channel config is provided.
    ( @step $idx:expr, $vec_channel_configs:expr, $app:expr, $head_type:ty => $head_channel_config:expr) => {

        $vec_channel_configs.push($head_channel_config);

        $app.add_event::<bevy_client_server_events::server::SendToClient<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::server::server_sends_messages_to_clients::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Server>()),
        );

        $app.add_event::<bevy_client_server_events::server::SendToClients<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::server::server_broadcasts_messages_to_clients::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Server>()),
        );

        $app.add_event::<bevy_client_server_events::server::ReceiveFromClient<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::server::server_receives_messages_from_clients::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Server>()),
        );

        $app.add_event::<bevy_client_server_events::client::SendToServer<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::client::client_sends_messages_to_server::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Client>()),
        );

        $app.add_event::<bevy_client_server_events::client::ReceiveFromServer<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::client::client_receives_messages_from_server::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Client>()),
        );
    };

    // For multiple type + channel configs.
    ( @step $idx:expr, $vec_channel_configs:expr, $app:expr, $head_type:ty => $head_channel_config:expr, $( $tail_type:ty => $tail_channel_config:expr ),* ) => {

        $vec_channel_configs.push($head_channel_config);

        $app.add_event::<bevy_client_server_events::server::SendToClient<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::server::server_sends_messages_to_clients::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Server>()),
        );

        $app.add_event::<bevy_client_server_events::server::SendToClients<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::server::server_broadcasts_messages_to_clients::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Server>()),
        );

        $app.add_event::<bevy_client_server_events::server::ReceiveFromClient<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::server::server_receives_messages_from_clients::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Server>()),
        );

        $app.add_event::<bevy_client_server_events::client::SendToServer<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::client::client_sends_messages_to_server::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Client>()),
        );

        $app.add_event::<bevy_client_server_events::client::ReceiveFromServer<$head_type>>().add_systems(
            bevy::prelude::PostUpdate,
            bevy::prelude::IntoSystemConfigs::run_if(bevy_client_server_events::client::client_receives_messages_from_server::<$idx, $head_type>, bevy::prelude::resource_exists::<bevy_client_server_events::Client>()),
        );

        bevy_client_server_events::paste::paste! {
            const [<$head_type:upper _IDX>]: u8 = $idx + 1; // Increment our index every type we iterate
            client_server_events_plugin!(@step [<$head_type:upper _IDX>], $vec_channel_configs, $app, $($tail_type => $tail_channel_config),*);
        }
    };

    // Entry point for the macro.
    ( $app:expr, $( $type:ty => $channel_config:expr ),* ) => {
        const START: u8 = 0; // Start at channel index 0
        let mut vec_channel_configs = Vec::new();
        client_server_events_plugin!(@step START, vec_channel_configs, $app, $($type => $channel_config),*);
        $app.add_plugins(
            bevy_client_server_events::ClientServerEventsPlugin {
                channels_config: bevy_client_server_events::NetworkConfigs(vec_channel_configs),
            }
        );
    };
}
