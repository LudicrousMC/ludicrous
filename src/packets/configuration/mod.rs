mod feature_flags;
mod finish_config;
mod known_packs;
mod plugin_message;
mod registry_data;
mod update_tags;
pub use feature_flags::FeatureFlags;
pub use finish_config::FinishConfig;
pub use known_packs::KnownPacks;
pub use plugin_message::PluginMessage;
pub use registry_data::RegistryData;
pub use update_tags::UpdateTags;

use super::{Packet, PacketStatic, PlayerReadConn, PlayerWriteConn};
use std::sync::{Arc, Weak};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex as AMutex;

pub struct InitialConfigurationPackets;

impl InitialConfigurationPackets {
    pub async fn match_and_send(id: i32, write_conn: &mut PlayerWriteConn) {
        match id {
            PluginMessage::CLIENTBOUND_ID => PluginMessage::new().send(write_conn).await,
            FeatureFlags::CLIENTBOUND_ID => FeatureFlags::new().send(write_conn).await,
            KnownPacks::CLIENTBOUND_ID => KnownPacks::new().send(write_conn).await,
            RegistryData::CLIENTBOUND_ID => RegistryData::new().send(write_conn).await,
            UpdateTags::CLIENTBOUND_ID => UpdateTags::new().send(write_conn).await,
            FinishConfig::CLIENTBOUND_ID => FinishConfig::new().send(write_conn).await,
            _ => {}
        }
    }

    pub async fn match_and_receive(id: i32, read_conn: &mut PlayerReadConn) {
        match id {
            FinishConfig::SERVERBOUND_ID => FinishConfig::new().receive(read_conn).await,
            _ => {}
        }
    }
}
