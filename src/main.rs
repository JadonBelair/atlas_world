use std::{io::BufReader, path::Path};

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use ahash::AHashMap;

const VIEWPORT_WIDTH: i32 = 320;
const VIEWPORT_HEIGHT: i32 = 180;

fn window_conf() -> Conf {
    let screen_size = if VIEWPORT_WIDTH as f32 * 0.7 < 640.0 {
        let ratio = VIEWPORT_WIDTH as f32 / VIEWPORT_HEIGHT as f32;
        ((640.0 / 0.7) as i32, (640.0 / 0.7 / ratio) as i32)
    } else {
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    };

    Conf {
        window_title: String::from("dugeon crawler"),
        window_resizable: true,
        window_width: screen_size.0,
        window_height: screen_size.1,
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Coords {
    h: i32,
    w: i32,
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Tile {
    atlas_coords: Coords,
    screen_coords: Coords,
    x: i32,
    z: i32,
    orientation: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Tiles {
    mode: i32,
    name: String,
    tiles: Vec<Tile>,
    r#type: i32,
}

#[derive(Serialize, Deserialize)]
struct AtlasInfo {
    layers: AHashMap<String, Tiles>,
}

struct Atlas {
    atlas_info: AtlasInfo,
    texture: Texture2D,
}

type AtlasCollection = AHashMap<String, Atlas>;
trait LoadAtlas {
    fn load<P: AsRef<Path>>(&mut self, atlas_id: &str, image_data: &[u8], data_path: P);
}

impl LoadAtlas for AtlasCollection {
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
}

struct Player {
    x: i32,
    y: i32,
    direction: i32,
}

#[derive(Deserialize)]
struct AtlasMap {
    width: usize,
    height: usize,
    wall: Vec<Vec<u8>>,
    floor: Vec<Vec<u8>>,
    ceiling: Vec<Vec<u8>>,
    object: Vec<Vec<u8>>,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut atlas = AtlasCollection::new();
    atlas.load("dungeon", include_bytes!("../mansion.png"), "mansion.json");
    atlas.load("common_objects", include_bytes!("../common_objects.png"), "common_objects.json");

    let mut player = Player {
        x: 1,
        y: 1,
        direction: 2,
    };

    let f = std::fs::File::open("out.json").unwrap();
    let file_buf = BufReader::new(f);
    let map = serde_json::from_reader(file_buf).unwrap();

    let render_depth = 9;
    let render_width = 22;

    let mut fullscreen = false;

    loop {
        clear_background(GRAY);

        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }

        if is_key_pressed(KeyCode::W) {
            move_forward(&mut player, &map);
        }
        if is_key_pressed(KeyCode::S) {
            move_backward(&mut player, &map);
        }

        if is_key_pressed(KeyCode::A) {
            strafe_left(&mut player, &map);
        }
        if is_key_pressed(KeyCode::D) {
            strafe_right(&mut player, &map);
        }

        if is_key_pressed(KeyCode::Q) {
            turn_left(&mut player);
        }
        if is_key_pressed(KeyCode::E) {
            turn_right(&mut player);
        }

        draw_rectangle(0.0, 0.0, screen_width() * 0.7, screen_height() * 0.7, BLACK);

        for z in -render_depth..1 {
            for x in (-render_width / 2)..=(render_width / 2) {
                draw_floor(&atlas, &player, &map, x, z)
            }
        }

        for z in -render_depth..1 {
            for x in (-render_width / 2)..=(render_width / 2) {
                draw_ceiling(&atlas, &player, &map, x, z);
            }
        }

        for z in -render_depth..1 {
            for x in (-render_width / 2)..0 {
                draw_map_square(&atlas, &player, &map, x, z);
            }
            for x in (1..=(render_width / 2)).rev() {
                draw_map_square(&atlas, &player, &map, x, z);
            }
            draw_map_square(&atlas, &player, &map, 0, z);
        }

        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            50.0,
            40.0,
            GREEN,
        );

        next_frame().await;
    }
}

fn get_player_direction_vector_offsets(player: &Player, x: i32, z: i32) -> IVec2 {
    if player.direction == 0 {
        return IVec2::new(player.x + x, player.y + z);
    } else if player.direction == 1 {
        return IVec2::new(player.x - z, player.y + x);
    } else if player.direction == 2 {
        return IVec2::new(player.x - x, player.y - z);
    } else if player.direction == 3 {
        return IVec2::new(player.x + z, player.y - x);
    }

    IVec2::NEG_ONE
}

fn get_tile_from_atlas(
    atlas: &AtlasCollection,
    atlas_id: &str,
    layer_id: &str,
    x: i32,
    z: i32,
    orientation: Option<String>,
) -> Option<Tile> {
    let layer = if let Some(atlas_info) = atlas.get(atlas_id) {
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
            if tile.orientation == orientation {
                return Some(tile.clone());
            }
        }
    }

    None
}

fn draw_floor(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    let p = get_player_direction_vector_offsets(player, x, z);

    if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        let map_value = map.floor[p.y as usize][p.x as usize];
        draw_tile(atlas, "dungeon", &format!("floor-{map_value}"), x, z, None);
    }
}

fn draw_ceiling(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    let p = get_player_direction_vector_offsets(player, x, z);

    if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        let map_value = map.ceiling[p.y as usize][p.x as usize];
        draw_tile(atlas, "dungeon", &format!("ceiling-{map_value}"), x, z, None);
    }
}

fn draw_map_square(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    let p = get_player_direction_vector_offsets(player, x, z);

    if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        if map.wall[p.y as usize][p.x as usize] != 0 {
            draw_side_walls(atlas, player, map, x, z);
            draw_front_walls(atlas, player, map, x, z);
        }

        if map.object[p.y as usize][p.x as usize] == 1 {
            draw_objects(atlas, player, map, x, z);
        }
    }
}

fn draw_side_walls(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    let p = get_player_direction_vector_offsets(player, x, z);

    if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        let wall_value = map.wall[p.y as usize][p.x as usize];
        if wall_value != 0 {
            draw_tile(atlas, "dungeon", &format!("wall-{wall_value}"), x, z, Some("left".to_owned()));
            draw_tile(atlas, "dungeon", &format!("wall-{wall_value}"), x, z, Some("right".to_owned()));
        }
    }
}

fn draw_front_walls(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    let p = get_player_direction_vector_offsets(player, x, z);

    if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        let wall_value = map.wall[p.y as usize][p.x as usize];
        if wall_value != 0 {
            draw_tile(atlas, "dungeon", &format!("wall-{wall_value}"), x, z, Some("front".to_owned()));
        }
    }
}

fn draw_objects(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    
	let p = get_player_direction_vector_offsets(player, x, z);
	
	if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        let map_value = map.object[p.y as usize][p.x as usize];
		if map_value != 0 {
            draw_tile(atlas, "common_objects", &format!("object-{map_value}"), x, z, None);
        }
    }
}

fn draw_tile(
    atlas: &AtlasCollection,
    atlas_id: &str,
    layer_id: &str,
    x: i32,
    z: i32,
    orientation: Option<String>,
) {
    let tile = get_tile_from_atlas(atlas, atlas_id, layer_id, x, z, orientation);

    let tex = if let Some(atlas_info) = atlas.get(atlas_id) {
        &atlas_info.texture
    } else {
        return;
    };

    if let Some(tile) = tile {
        let scale_x = screen_width() / VIEWPORT_WIDTH as f32 * 0.7;
        let scale_y = screen_height() / VIEWPORT_HEIGHT as f32 * 0.7;
        draw_texture_ex(
            tex,
            (tile.screen_coords.x as f32 * scale_x).ceil(),
            (tile.screen_coords.y as f32 * scale_y).ceil(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(scale_x * tile.screen_coords.w as f32 + 1.0, scale_y * tile.screen_coords.h as f32 + 1.0).ceil()),
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

fn can_move(map: &AtlasMap, pos: IVec2) -> bool {
	return map.wall[pos.y as usize][pos.x as usize] == 0
}

fn invert_direction(direction: i32) -> i32 {
    (direction + 2) % 4
}

fn get_dest_pos(player: &Player, direction: i32) -> IVec2 {
	
	let mut dest_vec = ivec2(
        ((direction*90) as f32).to_radians().sin() as i32,
		-(((direction*90) as f32).to_radians().cos() as i32)
	);

	dest_vec.x = dest_vec.x + player.x;
	dest_vec.y = dest_vec.y + player.y;
	
	return dest_vec;
}
				
fn move_forward(player: &mut Player, map: &AtlasMap) {

	let dest_pos = get_dest_pos(player, player.direction);

	if can_move(map, dest_pos) {
		player.x = dest_pos.x;
		player.y = dest_pos.y;
    }
}
	
fn move_backward(player: &mut Player, map: &AtlasMap) {

	let dest_pos = get_dest_pos(player, invert_direction(player.direction));

	if can_move(map, dest_pos) {
		player.x = dest_pos.x;
		player.y = dest_pos.y;
    }
}
		
fn strafe_left(player: &mut Player, map: &AtlasMap) {

	let mut direction = player.direction - 1;
	if direction < 0 {
        direction = 3;
    }
	
	let dest_pos = get_dest_pos(player, direction);

	if can_move(map, dest_pos) {
		player.x = dest_pos.x;
		player.y = dest_pos.y;
    }
}
	
fn strafe_right(player: &mut Player, map: &AtlasMap) {

	let direction = (player.direction + 1) % 4;
	
	let dest_pos = get_dest_pos(player, direction);

	if can_move(map, dest_pos) {
		player.x = dest_pos.x;
		player.y = dest_pos.y;
    }
}
			
fn turn_left(player: &mut Player) {
	player.direction = player.direction - 1;
	if player.direction < 0 {
		player.direction = 3;
    }
}
		
fn turn_right(player: &mut Player) {
	player.direction = player.direction + 1;
	if player.direction > 3 {
		player.direction = 0;
    }
}
