#![feature(thread_id_value)]
mod entities;
mod packets;
mod player;
mod server;
use crate::server::terrain_gen::noise_generator::initialize_noise_instances;
use packets::configuration::*;
use packets::handshake::HandshakeState;
use packets::play::*;
use packets::{Packet, PacketStatic};
use player::{PlayerState, PlayerStream};
use rsa::{pkcs8::EncodePublicKey, RsaPrivateKey};
use server::chunk_system::LudiChunkLoader;
use server::logger::{LogDomain, LogLevel, ServerLogger, LOGGER};
use server::randomness::{RandomGenerator, RandomPositionalGenerator};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::net::TcpListener;

pub const MC_VERSION: &str = "1.21.6";
pub const MC_PROTOCOL: usize = 771;
pub const JAR_RESOURCES_DIR: &str = "versions/1.21.6/minecraft";
/// The preset found in "versions/[version]/minecraft/worldgen/world_preset/"
pub const WORLD_PRESET: &str = "normal";
/// The density function to evaluate for dimension terrain generation
pub const MAIN_DENSITY_FUNCTION: &str = "final_density";
pub const MAX_BLOCKSTATES: usize = 27_946;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    LOGGER.get_or_init(ServerLogger::new);
    LOGGER
        .get()
        .unwrap()
        .println("Starting Ludicrous on MC Version 1.21.6...");
    process_eula();
    download_game_assets().await.unwrap();
    generate_server_encryption();
    let server = Arc::new(server::ServerData::new());

    once_cell::sync::Lazy::force(
        &server::terrain_gen::func_deserialize::EXTERNAL_DENSITY_FUNCTIONS,
    );
    initialize_noise_instances();

    LOGGER.get().unwrap().println("Loading Server Mappings...");
    let time = std::time::Instant::now();
    // Access mappings so they are loaded in memory and drop since they are not needed in this
    // scope
    once_cell::sync::Lazy::force(&server::chunk_system::BLOCKSTATE_MAPPINGS);
    once_cell::sync::Lazy::force(&server::chunk_system::BIOME_MAPPINGS);
    LOGGER.get().unwrap().println(&format!(
        "Finished Loading Server Mappings! ({:?})",
        std::time::Instant::now().duration_since(time)
    ));
    let listener = TcpListener::bind(format!("0.0.0.0:{}", server.config.server_port)).await?;
    LOGGER.get().unwrap().println_as(
        &format!(
            "Done! ({:?}) Listening on port {}...",
            std::time::Instant::now().duration_since(server.start_time),
            listener.local_addr().unwrap().port()
        ),
        LogDomain::Server,
        LogLevel::Info,
    );
    loop {
        let (socket, _addr) = listener.accept().await?;
        socket.set_nodelay(true)?;

        tokio::spawn(handle_client(socket, server.clone()));
    }
}

async fn handle_client(socket: tokio::net::TcpStream, server_data: Arc<server::ServerData>) {
    let mut handshake_state = HandshakeState::Status;
    let mut player_stream = PlayerStream::new(socket);
    // Handle handshake or begin Login process
    packets::handshake::Handshake::new(&mut handshake_state)
        .handle(&mut player_stream)
        .await;
    // Hold Player Packet tx until end of function to prevent closing rx
    let ((tx, mut rx), (_low_tx, mut low_rx)) = match handshake_state {
        HandshakeState::Status => {
            packets::status::Status::new()
                .handle(&mut player_stream)
                .await;
            packets::status::Ping::new()
                .handle(&mut player_stream)
                .await;
            return;
        }
        HandshakeState::Login => {
            // Start Login Stage and Initialize Player Packet Send Queue
            let mut login_pkt = packets::login::LoginStart::new(server_data.clone());
            login_pkt.handle(&mut player_stream).await;
            (
                login_pkt.high_channel.take().unwrap(),
                login_pkt.low_channel.take().unwrap(),
            )
        }
    };
    // Enable Encryption
    packets::login::Encryption::new()
        .handle(&mut player_stream)
        .await;
    // Enable Compression
    if server_data.config.network_compression_threshold > -1 {
        packets::login::SetCompression::new(server_data.config.network_compression_threshold)
            .handle(&mut player_stream)
            .await;
    }

    // Register player with server
    let player_data = player_stream.read.data.clone().unwrap().clone();
    server_data.add_player(player_data.clone());
    LOGGER.get().unwrap().println(&format!(
        "Registered player {} ({}) with entity id {}",
        player_data.username,
        player_data.get_uuid_string(),
        player_data.id
    ));

    // Add Authentication

    // Start Configuration Stage
    packets::login::LoginSuccess::new()
        .handle(&mut player_stream)
        .await;

    if player_data.state.load(Ordering::Relaxed) != PlayerState::Configuration as u8 {
        return;
    }
    let (mut player_read, mut player_write) = player_stream.split();

    // Player Packet Sender
    tokio::spawn({
        async move {
            // Send Configuration Packets
            let initial_config_packets = vec![
                PluginMessage::CLIENTBOUND_ID,
                FeatureFlags::CLIENTBOUND_ID,
                KnownPacks::CLIENTBOUND_ID,
                RegistryData::CLIENTBOUND_ID,
                FinishConfig::CLIENTBOUND_ID,
            ];

            for pkt in initial_config_packets {
                packets::configuration::InitialConfigurationPackets::match_and_send(
                    pkt,
                    &mut player_write,
                )
                .await;
            }

            // Listen for packets to send
            loop {
                tokio::select! {
                    biased;
                    Some(mut pkt) = rx.recv() => pkt.send(&mut player_write).await,
                    Some(mut pkt) = low_rx.recv() => pkt.send(&mut player_write).await,
                    else => break,
                }
            }
        }
    });

    // Player Packet Receiver
    loop {
        let packet = player_read.decrypt_packet().await;
        if packet.is_none() {
            break;
        }
        let mut packet = packet.unwrap();
        let id = packets::read_varint_from_vec(&mut packet);
        if let Some(id) = id {
            let state = PlayerState::from_u8(player_data.state.load(Ordering::Relaxed)).unwrap();
            match state {
                PlayerState::Configuration => {
                    if id == FinishConfig::SERVERBOUND_ID {
                        player_data
                            .state
                            .store(PlayerState::Play as u8, Ordering::Relaxed);
                        // Send Play Packets
                        let initial_login_packets = vec![
                            Login::new(server_data.clone()).into(),
                            SynchronizePlayerPos::new().into(),
                            PlayerInfoUpdate::new().into(),
                        ];

                        for pkt in initial_login_packets {
                            if tx.send(pkt).await.is_err() {
                                break;
                            }
                        }
                        server_data.load_init_chunks(player_data.clone(), 2).await;
                        let _ = tx.send(GameEvent::new(13, 0.0).into()).await;
                        server_data
                            .load_init_chunks(
                                player_data.clone(),
                                player_data.server.config.view_distance,
                            )
                            .await;
                        let _ = tx
                            .send(PlayerAbilities::new(0b00000110, 3.0, 0.0).into())
                            .await;
                        let player = player_data.clone();
                        let server = server_data.clone();
                        tokio::spawn(async move {
                            while let Some(tx) = player.outbound.upgrade() {
                                let _ = tx.send(KeepAlive::new(server.clone()).into()).await;
                                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                            }
                        });
                    }
                }
                PlayerState::Play => {
                    packets::play::ReceivablePlayPackets::match_and_receive(
                        id,
                        &packet,
                        &mut player_read,
                    )
                    .await;
                }
                _ => {
                    // Disconnect if invalid state
                    break;
                }
            }
        }
    }
    server_data.remove_player(player_data.id);
    LOGGER.get().unwrap().println_as(
        &format!(
            "Disconnected player {} ({})",
            player_data.username,
            player_data.get_uuid_string()
        ),
        LogDomain::Network,
        LogLevel::Info,
    );
}

fn process_eula() {
    let eula_file = File::open("eula.txt");
    if let Err(e) = eula_file {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                let mut new_file =
                    File::create_new("eula.txt").unwrap_or_else(|_| panic!("Error creating eula"));
                let time_parser = time::format_description::parse("[weekday repr:short] [month repr:short] [day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute] [year]").unwrap();
                let date_time = time::OffsetDateTime::now_local()
                    .unwrap_or(time::OffsetDateTime::now_utc())
                    .format(&time_parser)
                    .unwrap();
                let mut data = Vec::new();
                data.extend(b"#By changing the setting below to TRUE you are indicating your agreement to Mojang's EULA (https://account.mojang.com/documents/minecraft_eula).
#");
                data.extend(date_time.as_bytes());
                data.extend(
                    b"
eula=false
",
                );
                new_file.write_all(&data).unwrap();
            }
            _ => panic!("{e}"),
        }
    } else {
        let mut buf = String::new();
        eula_file.unwrap().read_to_string(&mut buf).unwrap();
        buf = buf
            .to_lowercase()
            .drain(
                buf.find("eula=")
                    .unwrap_or_else(|| panic!("Error: malformed eula"))..,
            )
            .collect::<String>();
        if buf.trim() == "eula=true" {
            return;
        }
    }
    LOGGER.get().unwrap().println_as(
        "You must agree to the eula.txt to continue...",
        LogDomain::Server,
        LogLevel::Warn,
    );
    std::process::exit(1);
}

async fn download_game_assets() -> Result<(), reqwest::Error> {
    if std::fs::read_dir(format!("versions/{MC_VERSION}")).is_ok() {
        return Ok(());
    }
    let manifest_res: serde_json::Value =
        reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
            .await?
            .json::<serde_json::Value>()
            .await?;
    let manifest_res = manifest_res.get("versions").unwrap().as_array().unwrap();
    let mut version_url = "";
    for v in manifest_res.iter() {
        if v.get("id").unwrap() == MC_VERSION {
            version_url = v.get("url").unwrap().as_str().unwrap();
        }
    }
    if version_url.is_empty() {
        panic!("Error: Could not find valid game assets for version {MC_VERSION}");
    }
    let version_res: serde_json::Value = reqwest::get(version_url)
        .await?
        .json::<serde_json::Value>()
        .await?;
    let asset_url = version_res
        .get("downloads")
        .unwrap()
        .get("client")
        .unwrap()
        .get("url")
        .unwrap()
        .as_str()
        .unwrap();
    LOGGER
        .get()
        .unwrap()
        .println(&format!("Retrieving game assets from {asset_url}"));
    let jar = std::io::Cursor::new(reqwest::get(asset_url).await?.bytes().await?.to_vec());
    let mut archive = zip::ZipArchive::new(jar).unwrap();
    let worldgen_output = "versions/".to_owned() + MC_VERSION + "/minecraft/worldgen";
    get_folder_from_archive(
        &mut archive,
        "data/minecraft/worldgen",
        std::path::Path::new(&worldgen_output),
    );
    let dim_type_output = "versions/".to_owned() + MC_VERSION + "/minecraft/dimension_type";
    get_folder_from_archive(
        &mut archive,
        "data/minecraft/dimension_type",
        std::path::Path::new(&dim_type_output),
    );
    Ok(())
}

fn get_folder_from_archive(
    archive: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
    path: &str,
    output_folder: &std::path::Path,
) {
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        if entry.name().starts_with(path) {
            let bare_path = entry.name().strip_prefix(path).unwrap();
            let bare_path = if let Some(new) = bare_path.strip_prefix('/') {
                new
            } else {
                bare_path
            };
            let entry_path = output_folder.join(bare_path);
            if entry.is_dir() {
                create_dir_all(entry_path).unwrap();
            } else {
                if let Some(parent_path) = entry_path.parent() {
                    create_dir_all(parent_path).unwrap();
                }
                let mut file = File::create_new(entry_path).unwrap();
                std::io::copy(&mut entry, &mut file).unwrap();
            }
        }
    }
}

fn generate_server_encryption() {
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 1024).unwrap();
    let public_key = private_key
        .to_public_key()
        .to_public_key_der()
        .unwrap()
        .as_bytes()
        .to_vec();

    let _ = packets::ENCRYPTION_DATA.set(packets::EncryptionData {
        public_key,
        private_key,
    });
}
