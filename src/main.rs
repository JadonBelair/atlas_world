use std::io::BufReader;

use atlas_world::*;
use macroquad::{prelude::*, ui::{root_ui, Skin, hash}};

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("dugeon crawler"),
        window_resizable: true,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut atlas = AtlasCollection::new();
    atlas.load("dungeon", include_bytes!("../mansion.png"), "mansion.json");
    atlas.load("common_objects", include_bytes!("../common_objects.png"), "common_objects.json");

    let player = Player {
        x: 1,
        y: 1,
        direction: 2,
    };
    
    let mut show_map = false;

    let f = std::fs::File::open("map.json").unwrap();
    let file_buf = BufReader::new(f);
    let map: AtlasMap = serde_json::from_reader(file_buf).unwrap();
    let mut auto_map = vec![vec![false; map.width]; map.height];

    let render_depth = 9;
    let render_width = 22;

    let mut fullscreen = false;

    let screen = render_target(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32);
    screen.texture.set_filter(FilterMode::Nearest);

    let zoom = vec2(1.0 / VIEWPORT_WIDTH as f32 * 2.0, 1.0 / VIEWPORT_HEIGHT as f32 * 2.0);

    let font = load_ttf_font("./assets/Minecraft.ttf").await.unwrap();

    let skin = {
        let button_style = root_ui()
            .style_builder()
            .background(load_image("./assets/button.png").await.unwrap())
            .background_margin(RectOffset { left: 6.0, right: 6.0, bottom: 6.0, top: 6.0 })
            .color(WHITE)
            .color_hovered(Color::new(0.75, 0.75, 0.75, 1.0))
            .color_selected_hovered(WHITE)
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

        let label_style = root_ui()
            .style_builder()
            .font_size(26)
            .font(include_bytes!("../assets/Minecraft.ttf")).unwrap()
            .build();
            
        Skin {
            button_style,
            window_style,
            window_titlebar_style,
            label_style,
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
    let sword_texture = Texture2D::from_rgba8(sword_image.width() as u16, sword_image.height() as u16, &sword_image.to_rgba8());

    let shield_image = image::open("./assets/shield_icon.png").unwrap();
    let shield_texture = Texture2D::from_rgba8(shield_image.width() as u16, shield_image.height() as u16, &shield_image.to_rgba8());

    let parry_image = image::open("./assets/parry_icon.png").unwrap();
    let parry_texture = Texture2D::from_rgba8(parry_image.width() as u16, parry_image.height() as u16, &parry_image.to_rgba8());

    let charge_image = image::open("./assets/charge_icon.png").unwrap();
    let charge_texture = Texture2D::from_rgba8(charge_image.width() as u16, charge_image.height() as u16, &charge_image.to_rgba8());

    let forward_image = image::open("./assets/forward_arrow.png").unwrap();
    let forward_texture = Texture2D::from_rgba8(forward_image.width() as u16, forward_image.height() as u16, &forward_image.to_rgba8());
    let back_texture = Texture2D::from_rgba8(forward_image.width() as u16, forward_image.height() as u16, &forward_image.flipv().to_rgba8());
    let right_texture = Texture2D::from_rgba8(forward_image.height() as u16, forward_image.width() as u16, &forward_image.rotate90().to_rgba8());
    let left_texture = Texture2D::from_rgba8(forward_image.height() as u16, forward_image.width() as u16, &forward_image.rotate270().to_rgba8());

    let turn_image = image::open("./assets/turn_arrow.png").unwrap();
    let turn_left_texture = Texture2D::from_rgba8(turn_image.width() as u16, turn_image.height() as u16, &turn_image.to_rgba8());
    let turn_right_texture = Texture2D::from_rgba8(turn_image.width() as u16, turn_image.height() as u16, &turn_image.fliph().to_rgba8());

    let map_image = image::open("./assets/map_icon.png").unwrap();
    let map_texture = Texture2D::from_rgba8(map_image.width() as u16, map_image.height() as u16, &map_image.to_rgba8());

    let background_image = image::open("./assets/background.png").unwrap();
    let background_texture = Texture2D::from_rgba8(background_image.width() as u16, background_image.height() as u16, &background_image.to_rgba8());

    let gl = unsafe { get_internal_gl() };
    let ctx = gl.quad_context;
    ctx.texture_set_wrap(background_texture.raw_miniquad_id(), miniquad::TextureWrap::Repeat, miniquad::TextureWrap::Repeat);

    let mut world = AtlasWorld {
        player,
        map,
        collection: atlas,
        render_depth,
        render_width
    };

    loop {
        set_camera(&viewport_camera);

        clear_background(BLACK);

        if !auto_map[world.player.y as usize][world.player.x as usize] {
            auto_map[world.player.y as usize][world.player.x as usize] = true;
        }

        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }

        if is_key_pressed(KeyCode::W) {
            world.player.move_forward(&world.map);
        }
        if is_key_pressed(KeyCode::S) {
            world.player.move_backward(&world.map);
        }

        if is_key_pressed(KeyCode::A) {
            world.player.strafe_left(&world.map);
        }
        if is_key_pressed(KeyCode::D) {
            world.player.strafe_right(&world.map);
        }

        if is_key_pressed(KeyCode::Q) {
            world.player.turn_left();
        }
        if is_key_pressed(KeyCode::E) {
            world.player.turn_right();
        }

        if is_key_pressed(KeyCode::M) {
            show_map = !show_map;
        }

        world.render();

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
            ..Default::default()
        });

        // draws the border around the viewport
        macroquad::ui::widgets::Window::new(hash!(), vec2(10.0, 10.0), dest_size).movable(false).close_button(false).ui(&mut root_ui(), |_| {});

        let map_size = dest_size * 0.9;
        let map_pos = vec2(10.0 + dest_size.x * 0.05, 10.0 + dest_size.y * 0.05);

        if show_map {
            // map border
            macroquad::ui::widgets::Window::new(hash!(), map_pos, map_size).movable(false).close_button(false).ui(&mut root_ui(), |_| {});

            draw_rectangle(map_pos.x, map_pos.y, map_size.x, map_size.y, BLACK);
            let map_pos = map_pos + 18.0;

            // let total_cells = vec2( (map_size.x - 36.0) / 15.0, (map_size.y - 36.0) / 15.0).floor();
            let total_cells = vec2(50.0, 40.0);
            let cell_size = ((map_size.x - 36.0) / total_cells.x).min((map_size.y - 36.0) / total_cells.y);

            let mut start_y = (world.player.y - (total_cells.y / 2.0) as i32).max(0);
            let mut end_y = (start_y + total_cells.y as i32).min(world.map.height as i32);
            if (start_y..end_y).len() < total_cells.y as usize {
                let diff = total_cells.y as i32 - (start_y..end_y).len() as i32;
                start_y -= diff;
                end_y += diff;
            }

            let mut start_x = (world.player.x - (total_cells.x / 2.0) as i32).max(0);
            let mut end_x = (start_x + total_cells.x as i32).min(world.map.width as i32);
            if (start_x..end_x).len() < total_cells.x as usize {
                let diff = total_cells.x as i32 - (start_x..end_x).len() as i32;
                start_x -= diff;
                end_x += diff;
            }

            let mut draw_y = 0;
            for y in start_y..end_y {
                let mut draw_x = 0;
                for x in start_x..end_x {
                    if x >= 0 && y >= 0 && x < world.map.width as i32 && y < world.map.height as i32 {
                        if auto_map[y as usize][x as usize] {
                            draw_rectangle(map_pos.x as f32 + (cell_size * draw_x as f32), map_pos.y as f32 + (cell_size * draw_y as f32), cell_size, cell_size, GRAY);
                            if x >= 0 {
                                if x == 0 || world.map.wall[y as usize][x as usize - 1] != 0 {
                                    draw_line(map_pos.x as f32 + (cell_size * draw_x as f32), map_pos.y as f32 + (cell_size * draw_y as f32), map_pos.x as f32 + (cell_size * draw_x as f32), map_pos.y as f32 + (cell_size * draw_y as f32) + cell_size, cell_size / 5.0, WHITE);
                                }
                            }
                            if x <= world.map.width as i32 - 1 {
                                if x == world.map.width as i32 - 1 || world.map.wall[y as usize][x as usize + 1] != 0 {
                                    draw_line(map_pos.x as f32 + (cell_size * draw_x as f32) + cell_size, map_pos.y as f32 + (cell_size * draw_y as f32), map_pos.x as f32 + (cell_size * draw_x as f32) + cell_size, map_pos.y as f32 + (cell_size * draw_y as f32) + cell_size, cell_size / 5.0, WHITE);
                                }
                            }
                            if y >= 0 {
                                if y == 0 || world.map.wall[y as usize - 1][x as usize] != 0 {
                                    draw_line(map_pos.x as f32 + (cell_size * draw_x as f32), map_pos.y as f32 + (cell_size * draw_y as f32), map_pos.x as f32 + (cell_size * draw_x as f32) + cell_size, map_pos.y as f32 + (cell_size * draw_y as f32), cell_size / 5.0, WHITE);
                                }
                            }
                            if y <= world.map.height as i32 - 1{
                                if y == world.map.height as i32 - 1 || world.map.wall[y as usize + 1][x as usize] != 0 {
                                    draw_line(map_pos.x as f32 + (cell_size * draw_x as f32), map_pos.y as f32 + (cell_size * draw_y as f32) + cell_size, map_pos.x as f32 + (cell_size * draw_x as f32) + cell_size, map_pos.y as f32 + (cell_size * draw_y as f32) + cell_size, cell_size / 5.0, WHITE);
                                }
                            }
                            if world.map.object[y as usize][x as usize] != 0 {
                                draw_circle(map_pos.x as f32 + (cell_size * draw_x as f32) + cell_size / 2.0, map_pos.y as f32 + (cell_size * draw_y as f32) + cell_size / 2.0, cell_size / 4.0, WHITE);
                            }
                        }
                        if world.player.x == x && world.player.y == y {
                            draw_circle(map_pos.x as f32 + (cell_size * draw_x as f32) + cell_size / 2.0, map_pos.y as f32 + (cell_size * draw_y as f32) + cell_size / 2.0, cell_size / 3.0, GREEN);
                        }
                        draw_x += 1;
                    }
                }
                if y >= 0 {draw_y += 1;}
            }
        }

        let right_side = dest_size.x + 20.0;

        let mut button_size = ((screen_width() - right_side) - 10.0 - 36.0) / 3.0;
        if screen_height() - 10.0 - 36.0 - button_size < button_size * 4.0 + 20.0 + 36.0 {
            button_size = (screen_height() - 30.0 - 72.0) / 5.0;
        }

        let win_pos = vec2(right_side, (screen_height() - button_size * 4.0 - 10.0 - 36.0).max(20.0 + 36.0 + button_size));
        let win_size = vec2(button_size * 3.0 + 36.0, button_size * 4.0 + 36.0);

        let window_hash = hash!();
        macroquad::ui::widgets::Window::new(window_hash, win_pos, win_size).movable(false).close_button(false).ui(&mut root_ui(), |ui| {
            macroquad::ui::widgets::Button::new(charge_texture.clone()).position(vec2(button_size, 0.0)).size(vec2(button_size, button_size)).ui(ui);
            macroquad::ui::widgets::Button::new(sword_texture.clone()).position(vec2(0.0, button_size)).size(vec2(button_size, button_size)).ui(ui);
            macroquad::ui::widgets::Button::new(shield_texture.clone()).position(vec2(button_size, button_size)).size(vec2(button_size, button_size)).ui(ui);
            macroquad::ui::widgets::Button::new(parry_texture.clone()).position(vec2(button_size * 2.0, button_size)).size(vec2(button_size, button_size)).ui(ui);

            if macroquad::ui::widgets::Button::new(forward_texture.clone()).position(vec2(button_size, button_size * 2.0)).size(vec2(button_size, button_size)).ui(ui) {
                world.player.move_forward(&world.map);
            }
            if macroquad::ui::widgets::Button::new(turn_left_texture.clone()).position(vec2(0.0, button_size * 2.0)).size(vec2(button_size, button_size)).ui(ui) {
                world.player.turn_left();
            }
            if macroquad::ui::widgets::Button::new(turn_right_texture.clone()).position(vec2(button_size * 2.0, button_size * 2.0)).size(vec2(button_size, button_size)).ui(ui) {
                world.player.turn_right();
            }
            if macroquad::ui::widgets::Button::new(back_texture.clone()).position(vec2(button_size, button_size * 3.0)).size(vec2(button_size, button_size)).ui(ui) {
                world.player.move_backward(&world.map);
            }
            if macroquad::ui::widgets::Button::new(left_texture.clone()).position(vec2(0.0, button_size * 3.0)).size(vec2(button_size, button_size)).ui(ui) {
                world.player.strafe_left(&world.map);
            }
            if macroquad::ui::widgets::Button::new(right_texture.clone()).position(vec2(button_size * 2.0, button_size * 3.0)).size(vec2(button_size, button_size)).ui(ui) {
                world.player.strafe_right(&world.map);
            }

            if macroquad::ui::widgets::Button::new(map_texture.clone()).position(vec2(0.0, 0.0)).size(vec2(button_size, button_size)).ui(ui) {
                show_map = !show_map;
            }
        });

        let win_pos = vec2(right_side, 10.0);
        let win_size = vec2(win_size.x, button_size + 36.0);
        macroquad::ui::widgets::Window::new(hash!(), win_pos, win_size).movable(false).close_button(false).ui(&mut root_ui(), |ui| {
            let mut canvas = ui.canvas();
            let cursor = canvas.cursor();
            canvas.rect(Rect::new(cursor.x, cursor.y, button_size * 3.0, button_size / 4.0), Color::default(), GRAY);
            canvas.rect(Rect::new(cursor.x, cursor.y, button_size * 3.0 * (5.0 / 20.0), button_size / 4.0), Color::default(), RED);
            let text_size = measure_text("15/20", Some(&font), 26, 1.0);
            macroquad::ui::widgets::Label::new(&format!("15/20")).position(vec2(((win_size.x - 36.0) / 2.0) - (text_size.width / 2.0), button_size / 8.0 - (text_size.height / 2.0))).ui(ui);
        });
        root_ui().focus_window(window_hash);

        next_frame().await;
    }
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

