use super::super::super::server::Chunk;
use super::super::{
    create_pkt_header, prepend_len_as_varint, write_varint, Packet, PacketStatic, Player,
    PlayerWriteConn,
};
use std::sync::Arc;

pub struct ChunkLightData {
    raw_data: Option<Vec<Vec<u8>>>,
}

impl ChunkLightData {
    pub fn new(chunk_data: Vec<Arc<Chunk>>, player_data: Arc<Player>) -> Self {
        let mut raw_data = Vec::new();
        for chunk in chunk_data {
            let mut chunk_data_packet = write_varint(Self::CLIENTBOUND_ID);
            chunk_data_packet.extend(chunk.x.to_be_bytes()); // Chunk X
            chunk_data_packet.extend(chunk.z.to_be_bytes()); // Chunk Z

            // HeightMap
            chunk_data_packet.extend(write_varint(chunk.get_heightmap_count()));
            if let Some(world_surface) = &chunk.heightmaps.world_surface {
                chunk_data_packet.extend(write_varint(1)); // Heightmap type
                                                           // Heightmap length
                chunk_data_packet.extend(write_varint(world_surface.len() as i32));
                for value in world_surface.iter() {
                    chunk_data_packet.extend(value.to_be_bytes());
                }
            }
            if let Some(ocean_floor) = &chunk.heightmaps.ocean_floor {
                chunk_data_packet.extend(write_varint(3)); // Heightmap type
                                                           // Heightmap length
                chunk_data_packet.extend(write_varint(ocean_floor.len() as i32));
                for value in ocean_floor.iter() {
                    chunk_data_packet.extend(value.to_be_bytes());
                }
            }
            if let Some(motion_blocking) = &chunk.heightmaps.motion_blocking {
                chunk_data_packet.extend(write_varint(4)); // Heightmap Type
                                                           // Heightmap length
                chunk_data_packet.extend(write_varint(motion_blocking.len() as i32));
                // Heightmap values
                for value in motion_blocking.iter() {
                    chunk_data_packet.extend(value.to_be_bytes());
                }
            }
            if let Some(motion_blocking_nl) = &chunk.heightmaps.motion_blocking_no_leaves {
                chunk_data_packet.extend(write_varint(5)); // Heightmap type
                                                           // Heightmap length
                chunk_data_packet.extend(write_varint(motion_blocking_nl.len() as i32));
                for value in motion_blocking_nl.iter() {
                    chunk_data_packet.extend(value.to_be_bytes());
                }
            }

            let mut chunk_palette_data = Vec::new();
            let mut sky_light_mask: u64 = 0;
            let mut block_light_mask: u64 = 0;
            let mut empty_sky_light_mask: u64 = 0;
            let mut empty_block_light_mask: u64 = 0;
            let mut sky_light_arrays: Vec<u8> = Vec::new();
            let mut block_light_arrays: Vec<u8> = Vec::new();

            // Block States and Biomes
            for section in chunk.sections.iter() {
                let mut non_air_blocks: i16 = 0;
                // Based on entries in palette. More entries means more bits needed
                // block_states calculations:
                if let Some(block_data) = &section.block_states.data {
                    // If Palette is not single valued (implied by it having a data field)
                    let bits_per_block = std::cmp::max(
                        4,
                        (section.block_states.palette.len() as f64).log2().ceil() as usize,
                    );
                    let bitmask = (1u64 << bits_per_block) - 1;

                    // Calculate blocks in section
                    for i in 0..4096 {
                        let blocks_per_long = 64 / bits_per_block;
                        let long_index = i / blocks_per_long;
                        let bit_offset = (i % blocks_per_long) * bits_per_block;
                        let block_container = block_data[long_index] as u64;
                        let block = (block_container >> bit_offset) & bitmask;
                        if let Some(block) = section.block_states.palette.get(block as usize) {
                            if block.id != 0 {
                                non_air_blocks += 1;
                            }
                        } else {
                            println!("could not get chunk block {block}, idx: {i}, chunk-x: {}, chunk-z: {}, section-y: {}, bpe {bits_per_block}", chunk.x, chunk.z, section.y);
                        }
                    }
                    // Chunk section block count
                    chunk_palette_data.extend(non_air_blocks.to_be_bytes());

                    // Chunk section bits per entry
                    chunk_palette_data.push(bits_per_block as u8);

                    chunk_palette_data
                        .extend(write_varint(section.block_states.palette.len() as i32)); // Palette length

                    // Palette entries:
                    for palette_block in section.block_states.palette.iter() {
                        chunk_palette_data.extend(write_varint(palette_block.id as i32));
                    }

                    // Chunk block data
                    for long in block_data.iter() {
                        chunk_palette_data.extend(long.to_be_bytes());
                    }
                } else if let Some(block) = section.block_states.palette.first() {
                    // If Palette is single valued (implied by it not having a data field)
                    // Blocks in these sections are already know since they are all the first
                    // palette entry
                    if block.id != 0 {
                        // Block count = 4096 since all of these blcoks are non air
                        chunk_palette_data.extend(4096i16.to_be_bytes());
                    } else {
                        // Block count = 0 since all of these blocks are air
                        chunk_palette_data.extend(0i16.to_be_bytes());
                    }

                    // Chunk section bits per entry (0 because single palette)
                    chunk_palette_data.push(0x00);

                    // Single valued palette block id
                    chunk_palette_data.extend(write_varint(block.id as i32));
                } else {
                    println!("could not get single palette chunk block, chunk-x: {}, chunk-z: {}, section-y: {}", chunk.x, chunk.z, section.y);
                    break;
                }

                // biomes calculators
                if let Some(biome_data) = &section.biomes.data {
                    // If palette is not single valued (implied by it having a data field)
                    let bits_per_biome =
                        (section.biomes.palette.len() as f64).log2().ceil().max(1.0) as u8;
                    let bitmask = (1u64 << bits_per_biome) - 1;

                    // Calculate biomes in section
                    // temp as single valued

                    // Biome palette bits per entry
                    chunk_palette_data.push(bits_per_biome);

                    // Biome Palette length
                    chunk_palette_data.extend(write_varint(section.biomes.palette.len() as i32));

                    // Biome palette biome ids
                    for biome in section.biomes.palette.iter() {
                        chunk_palette_data.extend(write_varint(*biome as i32));
                        // id 0 as placeholder
                        // biome (don't have mappings yet)
                    }
                    for long in biome_data.iter() {
                        chunk_palette_data.extend(long.to_be_bytes());
                    }
                    //chunk_palette_data.push(0x00);
                    //chunk_palette_data.extend(write_varint(0));
                } else if let Some(biome) = section.biomes.palette.first() {
                    // Biome palette bits per entry (0 since no data field)
                    chunk_palette_data.push(0x00);

                    // Biome id for palette
                    chunk_palette_data.extend(write_varint(*biome as i32)); // 0 as placeholder cause I don't
                                                                            // have mappings yet
                } else {
                    println!("biome palette error for chunk section");
                    break;
                }

                let bit = 1u64 << (section.y + 4) as u8;
                if let Some(sky_light) = &section.sky_light {
                    sky_light_mask |= bit;
                    sky_light_arrays.extend(write_varint(2048));
                    sky_light_arrays
                        .extend(sky_light.iter().map(|k| *k as u8).collect::<Vec<u8>>());
                } else {
                    empty_sky_light_mask |= bit;
                }

                if let Some(block_light) = &section.block_light {
                    block_light_mask |= bit;
                    block_light_arrays.extend(write_varint(2048));
                    block_light_arrays
                        .extend(block_light.iter().map(|k| *k as u8).collect::<Vec<u8>>());
                } else {
                    empty_block_light_mask |= bit;
                }
            }

            // Format chunk data length
            prepend_len_as_varint(&mut chunk_palette_data);
            chunk_data_packet.extend(&chunk_palette_data);

            chunk_data_packet.extend(write_varint(0)); // Block entities

            // Chunk Light data
            chunk_data_packet.push(0x01); // long array of sky light mask
            chunk_data_packet.extend(sky_light_mask.to_be_bytes());

            chunk_data_packet.push(0x01); // long array of block light mask
            chunk_data_packet.extend(block_light_mask.to_be_bytes());

            chunk_data_packet.push(0x01); // empty sky light bitset
            chunk_data_packet.extend(empty_sky_light_mask.to_be_bytes());

            chunk_data_packet.push(0x01); // empty block light bitset
            chunk_data_packet.extend(empty_block_light_mask.to_be_bytes());

            chunk_data_packet.extend(write_varint((sky_light_arrays.len() / 2048) as i32));
            chunk_data_packet.extend(sky_light_arrays.clone());

            chunk_data_packet.extend(write_varint((block_light_arrays.len() / 2048) as i32));
            chunk_data_packet.extend(block_light_arrays.clone());

            /*println!(
                "sky light {}, block light {}",
                sky_light_arrays.len(),
                block_light_arrays.len()
            );*/
            raw_data.push(create_pkt_header(
                &mut chunk_data_packet,
                Some(player_data.clone()),
            ));
            raw_data.push(chunk_data_packet);
        }
        ChunkLightData {
            raw_data: Some(raw_data),
        }
    }
}

impl PacketStatic for ChunkLightData {
    const CLIENTBOUND_ID: i32 = 0x27;
}

#[async_trait::async_trait]
impl Packet for ChunkLightData {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        // bundle delim
        write_conn.write_packet(write_varint(0x00)).await;
        for pkt in self.raw_data.take().unwrap() {
            write_conn.write_packet_data(pkt).await;
        }
        //bundle delim
        write_conn.write_packet(write_varint(0x00)).await;
    }
}
