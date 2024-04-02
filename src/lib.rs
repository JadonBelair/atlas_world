use std::{io::BufReader, path::Path};

use ahash::AHashMap;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

pub const VIEWPORT_WIDTH: i32 = 320;
pub const VIEWPORT_HEIGHT: i32 = 256;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Coords {
    pub h: i32,
    pub w: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile {
    pub atlas_coords: Coords,
    pub screen_coords: Coords,
    pub x: i32,
    pub z: i32,
    pub orientation: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Tiles {
    pub mode: i32,
    pub name: String,
    pub tiles: Vec<Tile>,
    pub r#type: i32,
}

#[derive(Serialize, Deserialize)]
pub struct AtlasInfo {
    pub layers: AHashMap<String, Tiles>,
}

pub struct Atlas {
    pub atlas_info: AtlasInfo,
    pub texture: Texture2D,
}

pub type AtlasCollection = AHashMap<String, Atlas>;
pub trait Collection {
    fn load<P: AsRef<Path>>(&mut self, atlas_id: &str, image_data: &[u8], data_path: P);
    fn get_tile(&self, atlas_id: &str, layer_id: &str, x: i32, z: i32, orientation: Option<String>) -> Option<Tile>;
}

impl Collection for AtlasCollection {
    fn load<P: AsRef<Path>>(&mut self, atlas_id: &str, image_data: &[u8], data_path: P) {
        let f = std::fs::File::open(data_path).unwrap();
        let buf = BufReader::new(f);
        let atlas_info = serde_json::from_reader(buf).unwrap();
        let texture = Texture2D::from_file_with_format(image_data, None);
        texture.set_filter(FilterMode::Nearest);
        let atlas = Atlas {
            atlas_info,
            texture,
        };
        self.insert(atlas_id.to_owned(), atlas);
    }

    fn get_tile(&self, atlas_id: &str, layer_id: &str, x: i32, z: i32, orientation: Option<String>) -> Option<Tile> {
        let layer = if let Some(atlas_info) = self.get(atlas_id) {
            if let Some(layer) = atlas_info.atlas_info.layers.get(layer_id) {
                layer
            } else {
                return None
            }
        } else {
            return None;
        };

        for tile in &layer.tiles {
            if tile.x == x && tile.z == z {
                if tile.orientation.is_none() || tile.orientation == orientation {
                    return Some(tile.clone());
                }
            }
        }

        None
    }
}

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub direction: i32,
}

impl Player {
    fn get_direction_vector_offsets(&self, x: i32, z: i32) -> IVec2 {
        match self.direction {
            0 => IVec2::new(self.x + x, self.y + z),
            1 => IVec2::new(self.x - z, self.y + x),
            2 => IVec2::new(self.x - x, self.y - z),
            3 => IVec2::new(self.x + z, self.y - x),
            _ => IVec2::NEG_ONE
        }
    }

    pub fn can_move(&self, map: &AtlasMap, pos: IVec2) -> bool {
        return (pos.x >= 0 && pos.y >= 0 && pos.x < map.width as i32 && pos.y < map.height as i32) && map.wall[pos.y as usize][pos.x as usize] == 0
    }

    pub fn invert_direction(&self) -> i32 {
        (self.direction + 2) % 4
    }

    pub fn get_dest_pos(&self, direction: i32) -> IVec2 {

        let mut dest_vec = ivec2(
            ((direction*90) as f32).to_radians().sin() as i32,
            -(((direction*90) as f32).to_radians().cos() as i32)
        );

        dest_vec.x = dest_vec.x + self.x;
        dest_vec.y = dest_vec.y + self.y;

        return dest_vec;
    }

    pub fn move_forward(&mut self, map: &AtlasMap) {

        let dest_pos = self.get_dest_pos(self.direction);

        if self.can_move(map, dest_pos) {
            self.x = dest_pos.x;
            self.y = dest_pos.y;
        }
    }

    pub fn move_backward(&mut self, map: &AtlasMap) {

        let dest_pos = self.get_dest_pos(self.invert_direction());

        if self.can_move(map, dest_pos) {
            self.x = dest_pos.x;
            self.y = dest_pos.y;
        }
    }

    pub fn strafe_left(&mut self, map: &AtlasMap) {

        let mut direction = self.direction - 1;
        if direction < 0 {
            direction = 3;
        }

        let dest_pos = self.get_dest_pos(direction);

        if self.can_move(map, dest_pos) {
            self.x = dest_pos.x;
            self.y = dest_pos.y;
        }
    }

    pub fn strafe_right(&mut self, map: &AtlasMap) {

        let direction = (self.direction + 1) % 4;

        let dest_pos = self.get_dest_pos(direction);

        if self.can_move(map, dest_pos) {
            self.x = dest_pos.x;
            self.y = dest_pos.y;
        }
    }

    pub fn turn_left(&mut self) {
        self.direction = self.direction - 1;
        if self.direction < 0 {
            self.direction = 3;
        }
    }

    pub fn turn_right(&mut self) {
        self.direction = self.direction + 1;
        if self.direction > 3 {
            self.direction = 0;
        }
    }
}

#[derive(Deserialize)]
pub struct AtlasMap {
    pub width: usize,
    pub height: usize,
    pub wall: Vec<Vec<u8>>,
    pub floor: Vec<Vec<u8>>,
    pub ceiling: Vec<Vec<u8>>,
    pub object: Vec<Vec<u8>>,
}

pub struct AtlasWorld {
    pub player: Player,
    pub map: AtlasMap,
    pub collection: AtlasCollection,
    pub render_depth: i32,
    pub render_width: i32,
}

impl AtlasWorld {
    pub fn render(&self) {
        for z in -self.render_depth..1 {
            for x in (-self.render_width / 2)..0 {
                self.draw_map_square(x, z);
            }
            for x in (0..=(self.render_width / 2)).rev() {
                self.draw_map_square(x, z);
            }
        }
    }


    pub fn draw_map_square(&self, x: i32, z: i32) {
        let p = self.player.get_direction_vector_offsets(x, z);

        if p.x >= 0 && p.y >= 0 && p.x < self.map.width as i32 && p.y < self.map.height as i32 {
            if self.map.floor[p.y as usize][p.x as usize] != 0 {
                let map_value = self.map.floor[p.y as usize][p.x as usize];
                self.draw_tile("dungeon", &format!("floor-{map_value}"), x, z, None);
            }

            if self.map.ceiling[p.y as usize][p.x as usize] != 0 {
                let map_value = self.map.ceiling[p.y as usize][p.x as usize];
                self.draw_tile("dungeon", &format!("ceiling-{map_value}"), x, z, None);
            }

            if self.map.wall[p.y as usize][p.x as usize] != 0 {
                self.draw_side_walls(x, z);
                self.draw_front_walls(x, z);
            }

            if self.map.object[p.y as usize][p.x as usize] != 0 {
                self.draw_objects(x, z);
            }
        }
    }

    pub fn draw_side_walls(&self, x: i32, z: i32) {
        let p = self.player.get_direction_vector_offsets(x, z);

        if p.x >= 0 && p.y >= 0 && p.x < self.map.width as i32 && p.y < self.map.height as i32 {
            let wall_value = self.map.wall[p.y as usize][p.x as usize];
            if wall_value != 0 {
                self.draw_tile("dungeon", &format!("wall-{wall_value}"), x, z, Some("left".to_owned()));
                self.draw_tile("dungeon", &format!("wall-{wall_value}"), x, z, Some("right".to_owned()));
            }
        }
    }

    pub fn draw_front_walls(&self, x: i32, z: i32) {
        let p = self.player.get_direction_vector_offsets(x, z);

        if p.x >= 0 && p.y >= 0 && p.x < self.map.width as i32 && p.y < self.map.height as i32 {
            let wall_value = self.map.wall[p.y as usize][p.x as usize];
            if wall_value != 0 {
                self.draw_tile("dungeon", &format!("wall-{wall_value}"), x, z, Some("front".to_owned()));
            }
        }
    }

    pub fn draw_objects(&self, x: i32, z: i32) {

        let p = self.player.get_direction_vector_offsets(x, z);

        if p.x >= 0 && p.y >= 0 && p.x < self.map.width as i32 && p.y < self.map.height as i32 {
            let map_value = self.map.object[p.y as usize][p.x as usize];
            if map_value != 0 {
                let orientation = Some(match self.player.direction {
                    0 => "front",
                    1 => "right",
                    2 => "back",
                    3 => "left",
                    _ => unreachable!()
                }.to_owned());
                self.draw_tile("common_objects", &format!("object-{map_value}"), x, z, orientation);
            }
        }
    }

    pub fn draw_tile(
        &self,
        atlas_id: &str,
        layer_id: &str,
        x: i32,
        z: i32,
        orientation: Option<String>,
    ) {
        let tile = self.collection.get_tile(atlas_id, layer_id, x, z, orientation);

        let tex = if let Some(atlas_info) = self.collection.get(atlas_id) {
            &atlas_info.texture
        } else {
            return;
        };

        if let Some(tile) = tile {
            draw_texture_ex(
                tex,
                tile.screen_coords.x as f32,
                tile.screen_coords.y as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(tile.screen_coords.w as f32, tile.screen_coords.h as f32)),
                    source: Some(Rect::new(
                            tile.atlas_coords.x as f32,
                            tile.atlas_coords.y as f32,
                            tile.atlas_coords.w as f32,
                            tile.atlas_coords.h as f32,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}
