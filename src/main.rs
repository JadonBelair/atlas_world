use std::{io::BufReader, path::Path};

use macroquad::{prelude::*, ui::{root_ui, Skin, hash}};
use serde::{Deserialize, Serialize};
use ahash::AHashMap;

const VIEWPORT_WIDTH: i32 = 320;
const VIEWPORT_HEIGHT: i32 = 256;

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("dugeon crawler"),
        window_resizable: true,
        window_width: 1280,
        window_height: 720,
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

    let f = std::fs::File::open("map.json").unwrap();
    let file_buf = BufReader::new(f);
    let map = serde_json::from_reader(file_buf).unwrap();

    let render_depth = 9;
    let render_width = 22;

    let mut fullscreen = false;

    let screen = render_target(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32);
    screen.texture.set_filter(FilterMode::Nearest);

    let zoom = vec2(1.0 / VIEWPORT_WIDTH as f32 * 2.0, 1.0 / VIEWPORT_HEIGHT as f32 * 2.0);

    let skin = {
        let button_style = root_ui()
            .style_builder()
            .background(load_image("./assets/button.png").await.unwrap())
            .background_hovered(load_image("./assets/button_hover.png").await.unwrap())
            .background_clicked(load_image("./assets/button.png").await.unwrap())
            .background_margin(RectOffset { left: 6.0, right: 6.0, bottom: 6.0, top: 6.0 })
            .color(WHITE)
            .color_selected_hovered(WHITE)
            .color_hovered(WHITE)
            .color_selected(WHITE)
            .color_inactive(WHITE)
            .color_clicked(WHITE)
            .margin(RectOffset { left: 4.0, right: 4.0, bottom: 4.0, top: 4.0 })
            .build();

        let window_style = root_ui()
            .style_builder()
            .background(load_image("./assets/border.png").await.unwrap())
            .background_clicked(load_image("./assets/border.png").await.unwrap())
            .background_hovered(load_image("./assets/border.png").await.unwrap())
            .background_margin(RectOffset { left: 14.0, right: 14.0, bottom: 14.0, top: 14.0 })
            .margin(RectOffset { left: 4.0, right: 4.0, bottom: 4.0, top: 4.0 })
            .color(WHITE)
            .color_selected_hovered(WHITE)
            .color_hovered(WHITE)
            .color_selected(WHITE)
            .color_inactive(WHITE)
            .color_clicked(WHITE)
            .build();

        let scrollbar_style = root_ui()
            .style_builder()
            .color(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_hovered(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_clicked(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_selected(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_inactive(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_selected_hovered(Color::new(0.0, 0.0, 0.0, 0.0))
            .build();

        let window_titlebar_style = root_ui()
            .style_builder()
            .color(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_hovered(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_clicked(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_selected(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_inactive(Color::new(0.0, 0.0, 0.0, 0.0))
            .color_selected_hovered(Color::new(0.0, 0.0, 0.0, 0.0))
            .build();
            
        Skin {
            button_style,
            window_style,
            window_titlebar_style,
            scroll_width: 0.0,
            title_height: 0.0,
            scrollbar_handle_style: scrollbar_style,
            ..root_ui().default_skin()
        }
    };

    root_ui().push_skin(&skin);

    let viewport_camera = Camera2D {
        render_target: Some(screen.clone()),
        zoom,
        offset: vec2(-1.0, -1.0),
        ..Default::default()
    };

    let sword_image = image::open("./assets/sword_icon.png").unwrap();
    let mut sword_texture = Texture2D::from_rgba8(sword_image.width() as u16, sword_image.height() as u16, &sword_image.to_rgba8());

    let shield_image = image::open("./assets/shield_icon.png").unwrap();
    let mut shield_texture = Texture2D::from_rgba8(shield_image.width() as u16, shield_image.height() as u16, &shield_image.to_rgba8());

    let parry_image = image::open("./assets/parry_icon.png").unwrap();
    let mut parry_texture = Texture2D::from_rgba8(parry_image.width() as u16, parry_image.height() as u16, &parry_image.to_rgba8());

    let charge_image = image::open("./assets/charge_icon.png").unwrap();
    let mut charge_texture = Texture2D::from_rgba8(charge_image.width() as u16, charge_image.height() as u16, &charge_image.to_rgba8());

    let forward_image = image::open("./assets/forward_arrow.png").unwrap();
    let mut forward_texture = Texture2D::from_rgba8(forward_image.width() as u16, forward_image.height() as u16, &forward_image.to_rgba8());
    let mut back_texture = Texture2D::from_rgba8(forward_image.width() as u16, forward_image.height() as u16, &forward_image.flipv().to_rgba8());
    let mut right_texture = Texture2D::from_rgba8(forward_image.height() as u16, forward_image.width() as u16, &forward_image.rotate90().to_rgba8());
    let mut left_texture = Texture2D::from_rgba8(forward_image.height() as u16, forward_image.width() as u16, &forward_image.rotate270().to_rgba8());

    let turn_image = image::open("./assets/turn_arrow.png").unwrap();
    let mut turn_left_texture = Texture2D::from_rgba8(turn_image.width() as u16, turn_image.height() as u16, &turn_image.to_rgba8());
    let mut turn_right_texture = Texture2D::from_rgba8(turn_image.width() as u16, turn_image.height() as u16, &turn_image.fliph().to_rgba8());

    let mut scaled_image = sword_image.clone();

    let background_image = image::open("./assets/background.png").unwrap();
    let background_texture = Texture2D::from_rgba8(background_image.width() as u16, background_image.height() as u16, &background_image.to_rgba8());

    let gl = unsafe { get_internal_gl() };
    let ctx = gl.quad_context;
    ctx.texture_set_wrap(background_texture.raw_miniquad_id(), miniquad::TextureWrap::Repeat, miniquad::TextureWrap::Repeat);

    loop {
        set_camera(&viewport_camera);

        clear_background(BLACK);

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

        for z in -render_depth..1 {
            for x in (-render_width / 2)..0 {
                draw_map_square(&atlas, &player, &map, x, z);
            }
            for x in (0..=(render_width / 2)).rev() {
                draw_map_square(&atlas, &player, &map, x, z);
            }
        }


        set_default_camera();

        clear_background(GRAY);

        draw_texture_ex(&background_texture, 0.0, 0.0, WHITE, DrawTextureParams {
            source: Some(Rect::new(0.0, 0.0, screen_width(), screen_height())),
            ..Default::default()
        });

        let dest_size = if (screen_width() * 0.7 - 20.0) / (VIEWPORT_WIDTH as f32 / VIEWPORT_HEIGHT as f32) > screen_height() - 20.0 {
            vec2((screen_height() - 20.0) * (VIEWPORT_WIDTH as f32 / VIEWPORT_HEIGHT as f32), screen_height() - 20.0)
        } else {
            vec2(screen_width() * 0.7 - 20.0, (screen_width() * 0.7 - 20.0) / (VIEWPORT_WIDTH as f32 / VIEWPORT_HEIGHT as f32))
        };

        draw_texture_ex(&screen.texture, 10.0, 10.0, WHITE, DrawTextureParams {
            dest_size: Some(dest_size),
            // dest_size: Some(vec2(screen_width() * 0.7, screen_height() * 0.7)),
            ..Default::default()
        });
        // draws the border around the viewport
        // root_ui().window(hash!(), vec2(10.0, 10.0), dest_size, |_| {});
        macroquad::ui::widgets::Window::new(hash!(), vec2(10.0, 10.0), dest_size).movable(false).close_button(false).ui(&mut root_ui(), |_| {});

        draw_text(
            &format!("FPS: {}", get_fps()),
            20.0,
            50.0,
            40.0,
            GREEN,
        );

        let right_side = dest_size.x + 20.0;

        let mut new_size = (((screen_width() - right_side) - 10.0 - 36.0) / 3.0).max(20.0) as u32 - 20;
        if screen_height() < (new_size + 20) as f32 * 4.0 + 20.0 + 36.0 {
            new_size = ((screen_height() - 20.0 - 36.0) / 4.0).max(20.0) as u32 - 20;
        }
        if new_size != scaled_image.width() {
            scaled_image = sword_image.resize(new_size, new_size, image::imageops::FilterType::Nearest);
            sword_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.to_rgba8());
            scaled_image = shield_image.resize(new_size, new_size, image::imageops::FilterType::Nearest);
            shield_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.to_rgba8());
            scaled_image = parry_image.resize(new_size, new_size, image::imageops::FilterType::Nearest);
            parry_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.to_rgba8());
            scaled_image = charge_image.resize(new_size, new_size, image::imageops::FilterType::Nearest);
            charge_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.to_rgba8());

            scaled_image = forward_image.resize(new_size, new_size, image::imageops::FilterType::Nearest);
            forward_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.to_rgba8());
            back_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.flipv().to_rgba8());
            right_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.rotate90().to_rgba8());
            left_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.rotate270().to_rgba8());

            scaled_image = turn_image.resize(new_size, new_size, image::imageops::FilterType::Nearest);
            turn_left_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.to_rgba8());
            turn_right_texture = Texture2D::from_rgba8(scaled_image.width() as u16, scaled_image.height() as u16, &scaled_image.fliph().to_rgba8());
        }

        let button_size = new_size as f32 + 20.0;

        let win_pos = vec2(right_side, screen_height() - button_size * 4.0 - 10.0 - 36.0);
        let win_size = vec2(button_size * 3.0 + 36.0, button_size * 4.0 + 36.0);

        macroquad::ui::widgets::Window::new(hash!(), win_pos, win_size).movable(false).close_button(false).ui(&mut root_ui(), |ui| {
            ui.button(Some(vec2(button_size, 0.0)), charge_texture.clone());
            ui.button(Some(vec2(button_size * 2.0, button_size)), sword_texture.clone());
            ui.button(Some(vec2(button_size, button_size)), shield_texture.clone());
            ui.button(Some(vec2(0.0, button_size)), parry_texture.clone());

            if ui.button(Some(vec2(button_size, 2.0 * button_size)), forward_texture.clone()) {
                move_forward(&mut player, &map);
            }
            if ui.button(Some(vec2(0.0, 2.0 * button_size)), turn_left_texture.clone()) {
                turn_left(&mut player);
            }
            if ui.button(Some(vec2(button_size * 2.0, 2.0 * button_size)), turn_right_texture.clone()) {
                turn_right(&mut player);
            }
            if ui.button(Some(vec2(button_size, 3.0 * button_size)), back_texture.clone()) {
                move_backward(&mut player, &map);
            }
            if ui.button(Some(vec2(0.0, 3.0 * button_size)), left_texture.clone()) {
                strafe_left(&mut player, &map);
            }
            if ui.button(Some(vec2(button_size * 2.0, 3.0 * button_size)), right_texture.clone()) {
                strafe_right(&mut player, &map);
            }
        });


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
            if tile.orientation.is_some() {
                if tile.orientation == orientation {
                    return Some(tile.clone());
                }
            } else {
                return Some(tile.clone());
            }
        }
    }

    None
}

// fn draw_floor(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
//     let p = get_player_direction_vector_offsets(player, x, z);
// 
//     if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
//         let map_value = map.floor[p.y as usize][p.x as usize];
//         draw_tile(atlas, "dungeon", &format!("floor-{map_value}"), x, z, None);
//     }
// }

// fn draw_ceiling(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
//     let p = get_player_direction_vector_offsets(player, x, z);
// 
//     if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
//         let map_value = map.ceiling[p.y as usize][p.x as usize];
//         draw_tile(atlas, "dungeon", &format!("ceiling-{map_value}"), x, z, None);
//     }
// }

fn draw_map_square(atlas: &AtlasCollection, player: &Player, map: &AtlasMap, x: i32, z: i32) {
    let p = get_player_direction_vector_offsets(player, x, z);

    if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        if map.floor[p.y as usize][p.x as usize] != 0 {
            let map_value = map.floor[p.y as usize][p.x as usize];
            draw_tile(atlas, "dungeon", &format!("floor-{map_value}"), x, z, None);
        }
        
        if map.ceiling[p.y as usize][p.x as usize] != 0 {
            let map_value = map.ceiling[p.y as usize][p.x as usize];
            draw_tile(atlas, "dungeon", &format!("ceiling-{map_value}"), x, z, None);
        }

        if map.wall[p.y as usize][p.x as usize] != 0 {
            draw_side_walls(atlas, player, map, x, z);
            draw_front_walls(atlas, player, map, x, z);
        }

        if map.object[p.y as usize][p.x as usize] != 0 {
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
	
    // println!("{x} {z}");

	if p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 {
        let map_value = map.object[p.y as usize][p.x as usize];
		if map_value != 0 {
            let orientation = Some(match player.direction {
                0 => "front",
                1 => "right",
                2 => "back",
                3 => "left",
                _ => unreachable!()
            }.to_owned());
            draw_tile(atlas, "common_objects", &format!("object-{map_value}"), x, z, orientation);
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
