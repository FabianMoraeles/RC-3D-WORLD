#![allow(unused_imports)]
#![allow(dead_code)]

mod line;
mod framebuffer;
mod maze;
mod caster;
mod player;

use framebuffer::Framebuffer;
use maze::{Maze, load_maze};
use caster::cast_ray;
use player::Player;

use image::{ImageReader, GenericImageView};

use raylib::prelude::*;
use raylib::core::audio::{RaylibAudio, Sound};
use raylib::consts::{GamepadAxis, GamepadButton};

use std::f32::consts::PI;
use std::time::{Duration, Instant};

// ===== Helpers: cargar imágenes RGBA sin raylib (para texturas/sprites) =====
fn load_image_rgba(path: &str) -> (Vec<u8>, usize, usize) {
    let img = ImageReader::open(path)
        .unwrap_or_else(|_| panic!("No se pudo abrir {}", path))
        .decode()
        .unwrap_or_else(|_| panic!("No se pudo decodificar {}", path));
    let (w, h) = img.dimensions();
    (img.to_rgba8().into_raw(), w as usize, h as usize)
}

// Ubicar la celda 'g' y devolver su centro en coordenadas del mundo
fn find_goal_center(maze: &Maze, block_size: usize) -> Option<(f32, f32)> {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == 'g' {
                let x = (i * block_size) as f32 + block_size as f32 * 0.5;
                let y = (j * block_size) as f32 + block_size as f32 * 0.5;
                return Some((x, y));
            }
        }
    }
    None
}

// ===== Movimiento / colisión =====
fn can_move(to_x: f32, to_y: f32, maze: &Maze, block_size: usize) -> bool {
    if to_x < 0.0 || to_y < 0.0 { return false; }
    let i = (to_x as usize) / block_size;
    let j = (to_y as usize) / block_size;
    matches!(maze.get(j).and_then(|row| row.get(i)).copied(), Some(' ') | Some('g'))
}

fn process_events_enhanced(
    player: &mut Player,
    rl: &RaylibHandle,
    maze: &Maze,
    block_size: usize,
    footstep_sound: &Sound,
    last_footstep_time: &mut Instant,
) {
    const MOVE_SPEED: f32 = 5.0;
    const ROT_SPEED: f32 = PI / 50.0;
    const FOOTSTEP_INTERVAL: f32 = 0.4;

    // Teclado: rotación fina
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a += ROT_SPEED; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a -= ROT_SPEED; }

    // Dirección y perpendicular (para strafe)
    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let perp_x = -dir_y;
    let perp_y =  dir_x;

    // Entrada acumulada
    let mut move_x = 0.0;
    let mut move_y = 0.0;
    let mut is_moving = false;

    // Teclado: W/S
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        move_x += dir_x * MOVE_SPEED; move_y += dir_y * MOVE_SPEED; is_moving = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        move_x -= dir_x * MOVE_SPEED; move_y -= dir_y * MOVE_SPEED; is_moving = true;
    }
    // Teclado: A/D (invertidos según tu versión previa)
    if rl.is_key_down(KeyboardKey::KEY_A) {
        move_x -= perp_x * MOVE_SPEED; move_y -= perp_y * MOVE_SPEED; is_moving = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        move_x += perp_x * MOVE_SPEED; move_y += perp_y * MOVE_SPEED; is_moving = true;
    }

    // ===== Gamepad (pad 0) =====
    if rl.is_gamepad_available(0) {
        let deadzone = 0.20;

        // Stick izquierdo -> mover (Y hacia delante es negativo)
        let lx = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        let ly = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        let ax = if lx.abs() > deadzone { lx } else { 0.0 };
        let ay = if ly.abs() > deadzone { ly } else { 0.0 };

        // Combinar avance/retro (dir) + strafe (perp)
        move_x += (dir_x * -ay + perp_x * ax) * MOVE_SPEED;
        move_y += (dir_y * -ay + perp_y * ax) * MOVE_SPEED;
        if ax != 0.0 || ay != 0.0 { is_moving = true; }

        // Stick derecho X -> rotación
        let rx = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
        if rx.abs() > deadzone {
            player.a += rx * 0.05; // sensibilidad giro stick derecho
        }
    }

    // Aplicar movimiento con colisión
    let new_x = player.pos.x + move_x;
    let new_y = player.pos.y + move_y;

    let mut moved = false;
    if can_move(new_x, player.pos.y, maze, block_size) { player.pos.x = new_x; moved = true; }
    if can_move(player.pos.x, new_y, maze, block_size) { player.pos.y = new_y; moved = true; }

    // Sonido de pasos
    if moved && is_moving {
        let now = Instant::now();
        if (now - *last_footstep_time).as_secs_f32() > FOOTSTEP_INTERVAL {
            footstep_sound.play();
            *last_footstep_time = now;
        }
    }
}

// ===== 2D debug =====
fn cell_to_color(cell: char) -> Color {
    match cell {
        '+' => Color::BLUEVIOLET,
        '-' => Color::VIOLET,
        '|' => Color::VIOLET,
        'g' => Color::GREEN,
        _   => Color::WHITE,
    }
}
fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize, yo: usize, block_size: usize, cell: char,
) {
    if cell == ' ' { return; }
    framebuffer.set_current_color(cell_to_color(cell));
    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}
pub fn render_maze(
    framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player,
) {
    for (r, row) in maze.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            draw_cell(framebuffer, c * block_size, r * block_size, block_size, cell);
        }
    }
    framebuffer.set_current_color(Color::WHITESMOKE);
    let num_rays = 5;
    for i in 0..num_rays {
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        let _ = cast_ray(framebuffer, maze, player, a, block_size, true);
    }
}

// ===== Sprite billboard sobre la 'g' =====
fn draw_sprite_billboard(
    framebuffer: &mut Framebuffer,
    player: &Player,
    dir_x: f32, dir_y: f32,
    plane_x: f32, plane_y: f32,
    zbuffer: &[f32],
    sprite_rgba: &[u8], sw: usize, sh: usize,
    world_x: f32, world_y: f32,
    scale: f32,
) {
    let spr_x = world_x - player.pos.x;
    let spr_y = world_y - player.pos.y;

    let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
    let trans_x = inv_det * (dir_y * spr_x - dir_x * spr_y);
    let trans_y = inv_det * (-plane_y * spr_x + plane_x * spr_y);
    if trans_y <= 0.0001 { return; }

    let screen_w = framebuffer.width as i32;
    let screen_h = framebuffer.height as i32;
    let hh = framebuffer.height as f32 / 2.0;
    let dp = 70.0;

    let sprite_screen_x = (framebuffer.width as f32 / 2.0) * (1.0 + trans_x / trans_y);

    let mut sprite_h = (hh / trans_y) * dp * scale;
    let mut sprite_w = sprite_h;

    let mut draw_start_y = (hh - sprite_h / 2.0).round() as i32;
    let mut draw_end_y   = (hh + sprite_h / 2.0).round() as i32;
    let mut draw_start_x = (sprite_screen_x - sprite_w / 2.0).round() as i32;
    let mut draw_end_x   = (sprite_screen_x + sprite_w / 2.0).round() as i32;

    if draw_start_y < 0 { draw_start_y = 0; }
    if draw_end_y >= screen_h { draw_end_y = screen_h - 1; }
    if draw_start_x < 0 { draw_start_x = 0; }
    if draw_end_x >= screen_w { draw_end_x = screen_w - 1; }

    let span_y = (draw_end_y - draw_start_y).max(1) as f32;
    let span_x = (draw_end_x - draw_start_x).max(1) as f32;

    for stripe in draw_start_x..=draw_end_x {
        let z = trans_y;
        if stripe < 0 || stripe as usize >= zbuffer.len() { continue; }
        if z >= zbuffer[stripe as usize] { continue; }

        let tex_x = (((stripe - draw_start_x) as f32 / span_x) * sw as f32) as usize;
        if tex_x >= sw { continue; }

        for y in draw_start_y..=draw_end_y {
            let tex_y = (((y - draw_start_y) as f32 / span_y) * sh as f32) as usize;
            if tex_y >= sh { continue; }

            let idx = (tex_y * sw + tex_x) * 4;
            if idx + 3 >= sprite_rgba.len() { continue; }
            let a = sprite_rgba[idx + 3];
            if a < 16 { continue; }

            let r = sprite_rgba[idx];
            let g = sprite_rgba[idx + 1];
            let b = sprite_rgba[idx + 2];

            let shade = (1.0 / (1.0 + z * 0.01)).min(1.0);
            let sr = (r as f32 * shade) as u8;
            let sg = (g as f32 * shade) as u8;
            let sb = (b as f32 * shade) as u8;

            framebuffer.set_pixel_color(stripe as u32, y as u32, Color::new(sr, sg, sb, a));
        }
    }
}

// ===== Render 3D =====
fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    wall_rgba: &[u8], wall_w: usize, wall_h: usize,
    floor_rgba: &[u8], floor_w: usize, floor_h: usize,
    goal_pos: Option<(f32, f32)>,
    sprite_rgba: &[u8], sprite_w: usize, sprite_h: usize,
) {
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;
    let screen_h = framebuffer.height as i32;

    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let plane_scale = (player.fov * 0.5).tan();
    let plane_x = -dir_y * plane_scale;
    let plane_y =  dir_x * plane_scale;
    let pos_z = hh * 0.8;

    let mut zbuffer = vec![f32::INFINITY; num_rays as usize];

    for i in 0..num_rays {
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        let intersect = cast_ray(framebuffer, maze, player, a, block_size, false);

        let perp = (player.a - a).cos().abs().max(0.0001) * intersect.distance.max(0.0001);
        zbuffer[i as usize] = perp;

        let camera_x = 2.0 * (i as f32) / (num_rays as f32) - 1.0;
        let ray_dir_x = dir_x + plane_x * camera_x;
        let ray_dir_y = dir_y + plane_y * camera_x;

        // Paredes
        let distance = intersect.distance.max(0.0001);
        let distance_to_projection_plane = 70.0;
        let stake_height = (hh / distance) * distance_to_projection_plane;

        let mut top = (hh - stake_height / 2.0).round() as i32;
        let mut bottom = (hh + stake_height / 2.0).round() as i32;
        let hmax = framebuffer.height as i32 - 1;
        if top < 0 { top = 0; }
        if bottom > hmax { bottom = hmax; }

        let hit_x = player.pos.x + distance * a.cos();
        let hit_y = player.pos.y + distance * a.sin();
        let wall_x = if ((hit_x % block_size as f32) - block_size as f32 / 2.0).abs()
                   > ((hit_y % block_size as f32) - block_size as f32 / 2.0).abs() {
            (hit_y % block_size as f32) / block_size as f32
        } else {
            (hit_x % block_size as f32) / block_size as f32
        };
        let tex_x = ((wall_x.fract().abs() * wall_w as f32) as usize).min(wall_w - 1);

        if bottom >= top {
            for y in top..=bottom {
                let d = y - top;
                let tex_y = if bottom > top {
                    ((d as f32 / (bottom - top) as f32) * wall_h as f32) as usize
                } else { 0 }.min(wall_h - 1);

                let idx = (tex_y * wall_w + tex_x) * 4;
                if idx + 3 < wall_rgba.len() {
                    let r = wall_rgba[idx];
                    let g = wall_rgba[idx + 1];
                    let b = wall_rgba[idx + 2];
                    let a_ = wall_rgba[idx + 3];

                    let shade = (1.0 / (1.0 + distance * 0.01)).min(1.0);
                    framebuffer.set_pixel_color(
                        i, y as u32,
                        Color::new((r as f32 * shade) as u8, (g as f32 * shade) as u8, (b as f32 * shade) as u8, a_)
                    );
                }
            }
        }

        // Suelo
        for y in (bottom + 1)..screen_h {
            let p = (y as f32) - hh;
            if p.abs() < 0.1 { continue; }
            let row_dist = (pos_z / p).max(1.0);
            let world_x = player.pos.x + row_dist * ray_dir_x;
            let world_y = player.pos.y + row_dist * ray_dir_y;

            let texture_scale = 4.0;
            let tx = (world_x * texture_scale / block_size as f32).fract().abs();
            let ty = (world_y * texture_scale / block_size as f32).fract().abs();

            let u = ((tx * floor_w as f32) as usize).min(floor_w - 1);
            let v = ((ty * floor_h as f32) as usize).min(floor_h - 1);

            let idx = (v * floor_w + u) * 4;
            if idx + 3 < floor_rgba.len() {
                let r = floor_rgba[idx];
                let g = floor_rgba[idx + 1];
                let b = floor_rgba[idx + 2];
                let a_ = floor_rgba[idx + 3];

                let dim = (1.0 / (1.0 + row_dist * 0.001)).min(1.0);
                framebuffer.set_pixel_color(
                    i, y as u32,
                    Color::new((r as f32 * dim) as u8, (g as f32 * dim) as u8, (b as f32 * dim) as u8, a_)
                );
            }
        }
    }

    // Sprite animado sobre 'g'
    if let Some((gx, gy)) = goal_pos {
        let sprite_scale = 1.0;
        draw_sprite_billboard(
            framebuffer,
            player,
            dir_x, dir_y,
            plane_x, plane_y,
            &zbuffer,
            sprite_rgba, sprite_w, sprite_h,
            gx, gy,
            sprite_scale,
        );
    }
}

// ======= MAIN (menú / juego / victoria + gamepad) =======
fn main() {
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster - Gamepad Ready")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(60);

    // Audio
    let audio = RaylibAudio::init_audio_device().expect("Audio init");
    let footstep_sound: Sound = audio.new_sound("assets/footstep.wav").expect("footstep.wav");
    let mut music = audio
        .new_music("assets/background.ogg")
        .or_else(|_| audio.new_music("assets/background.mp3"))
        .expect("musica");
    music.set_volume(0.4);
    music.play_stream();

    // Framebuffer / mundo / jugador
    let mut fb = Framebuffer::new(window_width as u32, window_height as u32);
    fb.set_background_color(Color::BLACK);

    let maze = load_maze("maze.txt");
    let goal_center = find_goal_center(&maze, block_size);

    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    // Texturas
    let (wall_rgba, wall_w, wall_h) = load_image_rgba("assets/wall.png");
    let (floor_rgba, floor_w, floor_h) = load_image_rgba("assets/floor.png");

    // Imágenes de menú y victoria
    let lobby_tex = window.load_texture(&thread, "assets/lobby.png").expect("assets/lobby.png");
    let win_tex = window.load_texture(&thread, "assets/win.png").ok();

    // Sprites animados (sobre 'g')
    let (spr1_rgba, spr1_w, spr1_h) = load_image_rgba("assets/freddy1.png");
    let (spr2_rgba, spr2_w, spr2_h) = load_image_rgba("assets/freddy2.png");
    let mut use_spr1 = true;
    let mut last_anim = Instant::now();
    let anim_interval = Duration::from_millis(250);

    // Estados
    enum GameState { Menu, Playing, Win }
    let mut state = GameState::Menu;
    let mut mode_2d = false;
    let mut last_step = Instant::now();

    while !window.window_should_close() {
        music.update_stream();

        match state {
            GameState::Menu => {
                let start_pressed =
                    window.is_key_pressed(KeyboardKey::KEY_ENTER) ||
                    window.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT) || // START
                    window.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN); // A

                if start_pressed {
                    window.set_mouse_position(Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0));
                    state = GameState::Playing;
                    continue;
                }

                let mut d = window.begin_drawing(&thread);
                d.clear_background(Color::BLACK);

                let tw = lobby_tex.width() as f32;
                let th = lobby_tex.height() as f32;
                let sw = window_width as f32;
                let sh = window_height as f32;
                let scale = (sw / tw).min(sh / th);
                let dw = tw * scale; let dh = th * scale;
                let dx = (sw - dw) * 0.5; let dy = (sh - dh) * 0.5;

                d.draw_texture_pro(
                    &lobby_tex,
                    Rectangle::new(0.0, 0.0, tw, th),
                    Rectangle::new(dx, dy, dw, dh),
                    Vector2::new(0.0, 0.0), 0.0, Color::WHITE);

                let txt = "Presiona A";
                let size = 28;
                let wtxt = d.measure_text(txt, size);
                d.draw_text(txt, (window_width - wtxt) / 2, window_height - 60, size, Color::RAYWHITE);
                d.draw_fps(10, 10);
            }

            GameState::Playing => {
                if Instant::now() - last_anim >= anim_interval {
                    use_spr1 = !use_spr1;
                    last_anim = Instant::now();
                }

                fb.clear();

                let toggle_view =
                    window.is_key_pressed(KeyboardKey::KEY_M) ||
                    window.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN); // A
                if toggle_view { mode_2d = !mode_2d; }

                // Rotación con mouse (opcional si se usa gamepad)
                let mouse = window.get_mouse_position();
                let cx = window_width as f32 / 2.0;
                let dx = mouse.x - cx;
                if dx.abs() > 1.0 {
                    let sens = 0.003;
                    player.a += dx * sens;
                    window.set_mouse_position(Vector2::new(cx, mouse.y));
                }

                process_events_enhanced(
                    &mut player, &window, &maze, block_size, &footstep_sound, &mut last_step);

                // ¿pisa g?
                let ci = (player.pos.x as usize) / block_size;
                let cj = (player.pos.y as usize) / block_size;
                if matches!(maze.get(cj).and_then(|r| r.get(ci)).copied(), Some('g')) {
                    state = GameState::Win;
                    continue;
                }

                let (spr_rgba, sw, sh) = if use_spr1 {
                    (&spr1_rgba, spr1_w, spr1_h)
                } else {
                    (&spr2_rgba, spr2_w, spr2_h)
                };

                if mode_2d {
                    render_maze(&mut fb, &maze, block_size, &player);
                } else {
                    render_world(
                        &mut fb, &maze, block_size, &player,
                        &wall_rgba, wall_w, wall_h,
                        &floor_rgba, floor_w, floor_h,
                        goal_center,
                        spr_rgba, sw, sh,
                    );
                }

                fb.swap_buffers(&mut window, &thread);
            }

            GameState::Win => {
                let accept =
                    window.is_key_pressed(KeyboardKey::KEY_ENTER) ||
                    window.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT) || // START
                    window.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN); // A
                if accept {
                    state = GameState::Menu;
                    player.pos = Vector2::new(150.0, 150.0);
                    player.a = PI / 3.0;
                    continue;
                }

                let mut d = window.begin_drawing(&thread);
                d.clear_background(Color::BLACK);

                if let Some(ref tex) = win_tex {
                    let tw = tex.width() as f32;
                    let th = tex.height() as f32;
                    let sw = window_width as f32;
                    let sh = window_height as f32;
                    let scale = (sw / tw).min(sh / th);
                    let dw = tw * scale; let dh = th * scale;
                    let dx = (sw - dw) * 0.5; let dy = (sh - dh) * 0.5;

                    d.draw_texture_pro(
                        tex,
                        Rectangle::new(0.0, 0.0, tw, th),
                        Rectangle::new(dx, dy, dw, dh),
                        Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
                } else {
                    let title = "¡VICTORIA!";
                    let prompt = "Presiona A";
                    let ts = 48; let ps = 24;
                    let tw = d.measure_text(title, ts);
                    let pw = d.measure_text(prompt, ps);
                    d.draw_text(title, (window_width - tw)/2, window_height/2 - 40, ts, Color::RAYWHITE);
                    d.draw_text(prompt, (window_width - pw)/2, window_height/2 + 20, ps, Color::LIGHTGRAY);
                }

                d.draw_fps(10, 10);
            }
        }
    }

    music.stop_stream();
}

//HOLAAAAAAAAAAAAAAAAAA