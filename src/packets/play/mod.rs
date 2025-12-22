/*mod bundle_delimiter;
mod change_difficulty;*/
mod chunk_light_data;
mod game_event;
mod initialize_border;
mod keep_alive;
mod login;
mod ping_sb;
mod player_abilities;
mod player_info_update;
/*mod recipe_book_settings;
mod remove_entities;*/
mod set_center_chunk;
/*mod set_container_content;
mod set_entity_metadata;
mod set_entity_vel;*/
mod set_player_pos;
mod set_player_pos_rot;
/*mod set_player_rot;
mod set_spawn_pos;
mod spawn_entity;*/
mod synchronize_player_pos;
//mod teleport_entity;
mod unload_chunks;
/*mod update_entity_pos;
mod update_entity_pos_rot;
mod update_entity_rot;
pub use bundle_delimiter::BundleDelimiter;
pub use change_difficulty::ChangeDifficulty;*/
pub use chunk_light_data::ChunkLightData;
pub use game_event::GameEvent;
pub use initialize_border::InitializeBorder;
pub use keep_alive::KeepAlive;
pub use login::Login;
use ping_sb::PingSB;
pub use player_abilities::PlayerAbilities;
pub use player_info_update::PlayerInfoUpdate;
//pub use recipe_book_settings::RecipeBookSettings;
//pub use remove_entities::RemoveEntities;
pub use set_center_chunk::SetCenterChunk;
/*pub use set_container_content::SetContainerContent;
pub use set_entity_metadata::{SetEntityMetadata, SetEntityMetadataPayload};
pub use set_entity_vel::{SetEntityVel, SetEntityVelPayload};
pub use set_spawn_pos::SetSpawnPos;
pub use spawn_entity::{SpawnEntity, SpawnEntityPayload};*/
pub use synchronize_player_pos::SynchronizePlayerPos;
//pub use teleport_entity::{TeleportEntity, TeleportEntityPayload};
pub use unload_chunks::UnloadChunks;
use {
    set_player_pos::SetPlayerPos,
    set_player_pos_rot::SetPlayerPosRot, //set_player_rot::SetPlayerRot,
};
/*pub use {
    update_entity_pos::{UpdateEntityPos, UpdateEntityPosPayload},
    update_entity_pos_rot::{UpdateEntityPosRot, UpdateEntityPosRotPayload},
    update_entity_rot::{UpdateEntityRot, UpdateEntityRotPayload},
};*/

use super::{
    super::server::ServerData, Packet, PacketStatic, Player, PlayerReadConn, PlayerWriteConn,
};
use std::sync::{Arc, Weak};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex as AMutex;
pub struct ReceivablePlayPackets;

impl ReceivablePlayPackets {
    pub async fn match_and_receive(id: i32, packet: &[u8], read_conn: &mut PlayerReadConn) {
        match id {
            PingSB::SERVERBOUND_ID => {
                let ping = PingSB::new(packet.to_owned());
                let tx = read_conn.data.clone().unwrap().outbound.clone();
                if let Some(tx) = tx.upgrade() {
                    let _ = tx.try_send(ping.into());
                }
            }
            SetPlayerPosRot::SERVERBOUND_ID => {
                SetPlayerPosRot::new(packet).receive(read_conn).await
            }
            SetPlayerPos::SERVERBOUND_ID => SetPlayerPos::new(packet).receive(read_conn).await,
            //SetPlayerRot::SERVERBOUND_ID => SetPlayerRot::new(player, packet).receive().await,
            _ => {}
        };
    }
}

pub struct InitialPlayPackets;

impl InitialPlayPackets {
    pub async fn match_and_send(
        id: i32,
        server: Arc<ServerData>,
        write_conn: &mut PlayerWriteConn,
    ) {
        match id {
            PlayerInfoUpdate::CLIENTBOUND_ID => PlayerInfoUpdate::new().send(write_conn).await,
            /*ChangeDifficulty::CLIENTBOUND_ID => {
                ChangeDifficulty::new(player, socket_write).send().await
            }*/
            //InitializeBorder::CLIENTBOUND_ID => InitializeBorder::new().send(write_conn).await,
            /*SetSpawnPos::CLIENTBOUND_ID => {
                SetSpawnPos::new(player, socket_write, server).send().await
            }
            RecipeBookSettings::CLIENTBOUND_ID => {
                RecipeBookSettings::new(player, socket_write).send().await
            }
            SetContainerContent::CLIENTBOUND_ID => {
                SetContainerContent::new(player, socket_write).send().await
            }*/
            Login::CLIENTBOUND_ID => Login::new(server).send(write_conn).await,
            SynchronizePlayerPos::CLIENTBOUND_ID => {
                SynchronizePlayerPos::new().send(write_conn).await
            }
            _ => {}
        };
    }
}
