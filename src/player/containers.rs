use super::super::server::ServerMappings;
use super::PlayerNBT;

pub struct PlayerInventory {
    pub items: [Item; 47],
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            items: [Item::default(); 47],
        }
    }

    pub fn from_nbt(nbt: &PlayerNBT, item_mappings: &ServerMappings) -> Self {
        let mut inventory = Self::new();
        if let Some(equipment) = &nbt.equipment {
            if let Some(helmet) = &equipment.head {
                inventory.items[5].count = helmet.count;
                inventory.items[5].id = *item_mappings
                    .items_to_numerical
                    .get(helmet.id.split_once(':').unwrap().1)
                    .unwrap();
            }
            if let Some(chestplate) = &equipment.chest {
                inventory.items[6].count = chestplate.count;
                inventory.items[6].id = *item_mappings
                    .items_to_numerical
                    .get(chestplate.id.split_once(':').unwrap().1)
                    .unwrap();
            }
            if let Some(leggings) = &equipment.legs {
                inventory.items[7].count = leggings.count;
                inventory.items[7].id = *item_mappings
                    .items_to_numerical
                    .get(leggings.id.split_once(':').unwrap().1)
                    .unwrap();
            }
            if let Some(boots) = &equipment.feet {
                inventory.items[8].count = boots.count;
                inventory.items[8].id = *item_mappings
                    .items_to_numerical
                    .get(boots.id.split_once(':').unwrap().1)
                    .unwrap();
            }
            if let Some(offhand) = &equipment.offhand {
                inventory.items[45].count = offhand.count;
                inventory.items[45].id = *item_mappings
                    .items_to_numerical
                    .get(offhand.id.split_once(':').unwrap().1)
                    .unwrap()
            }
        }
        for (i, item) in inventory.items.iter_mut().enumerate() {
            if i < 9 {
                continue;
            }
            let slot = if i > 35 { i - 36 } else { i };
            nbt.inventory.iter().for_each(|item_nbt| {
                if item_nbt.slot.unwrap() == slot as u8 {
                    item.count = item_nbt.count;
                    item.id = *item_mappings
                        .items_to_numerical
                        .get(item_nbt.id.split_once(':').unwrap().1)
                        .unwrap()
                }
            })
        }
        inventory
    }
}

impl std::fmt::Debug for PlayerInventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Items {...}").finish()
    }
}

#[derive(Default, Clone, Copy)]
pub struct Item {
    pub count: i8,
    pub id: usize,
}

impl Item {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

// Formatter to use less screen space
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!(
            "Item {{ count: {}, id: {} }}",
            self.count, self.id
        ))
        .finish()
    }
}
