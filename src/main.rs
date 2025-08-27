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
use std::f32::consts::PI;

// ===== Helper para cargar imágenes (texturas de paredes/suelo) =====
fn load_image_rgba(path: &str) -> (Vec<u8>, usize, usize) {
    let img = ImageReader::open(path)
        .unwrap_or_else(|_| panic!("No se pudo abrir {}", path))
        .decode()
        .unwrap_or_else(|_| panic!("No se pudo decodificar {}", path));
    let (w, h) = img.dimensions();
    (img.to_rgba8().into_raw(), w as usize, h as usize)
}

// ===== Control de jugador / colisión =====
// Permitimos caminar sobre ' ' y también sobre 'g' (meta)
fn can_move(to_x: f32, to_y: f32, maze: &Maze, block_size: usize) -> bool {
    if to_x < 0.0 || to_y < 0.0 { return false; }
    let i = (to_x as usize) / block_size;
    let j = (to_y as usize) / block_size;
    matches!(maze.get(j).and_then(|row| row.get(i)).copied(), Some(' ') | Some('g'))
}

// Movimientos + pasos (audio)
fn process_events_enhanced(
    player: &mut Player,
    rl: &RaylibHandle,
    maze: &Maze,
    block_size: usize,
    footstep_sound: &Sound,
    last_footstep_time: &mut std::time::Instant,
) {
    const MOVE_SPEED: f32 = 5.0;
    const ROT_SPEED: f32 = PI / 50.0;
    const FOOTSTEP_INTERVAL: f32 = 0.4;

    // Rotación con flechas
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a += ROT_SPEED; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a -= ROT_SPEED; }

    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let perp_x = -dir_y;
    let perp_y =  dir_x;

    // Movimiento con WASD
    let mut move_x = 0.0;
    let mut move_y = 0.0;
    let mut is_moving = false;

    // W/S - adelante/atrás
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        move_x += dir_x * MOVE_SPEED;
        move_y += dir_y * MOVE_SPEED;
        is_moving = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        move_x -= dir_x * MOVE_SPEED;
        move_y -= dir_y * MOVE_SPEED;
        is_moving = true;
    }

    // A/D - strafe (invertidos como los tienes ahora)
    if rl.is_key_down(KeyboardKey::KEY_A) {
        move_x -= perp_x * MOVE_SPEED;
        move_y -= perp_y * MOVE_SPEED;
        is_moving = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        move_x += perp_x * MOVE_SPEED;
        move_y += perp_y * MOVE_SPEED;
        is_moving = true;
    }

    // Aplicar movimiento con colisión
    let new_x = player.pos.x + move_x;
    let new_y = player.pos.y + move_y;

    let mut actually_moved = false;
    if can_move(new_x, player.pos.y, maze, block_size) {
        player.pos.x = new_x;
        actually_moved = true;
    }
    if can_move(player.pos.x, new_y, maze, block_size) {
        player.pos.y = new_y;
        actually_moved = true;
    }

    // Reproducir pasos si se movió
    if actually_moved && is_moving {
        let now = std::time::Instant::now();
        if (now - *last_footstep_time).as_secs_f32() > FOOTSTEP_INTERVAL {
            footstep_sound.play();
            *last_footstep_time = now;
        }
    }
}

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
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
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
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
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

// ===== Render 3D con texturas =====
fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    wall_rgba: &[u8],
    wall_w: usize,
    wall_h: usize,
    floor_rgba: &[u8],
    floor_w: usize,
    floor_h: usize,
) {
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;
    let screen_h = framebuffer.height as i32;

    // Configuración de cámara para floor casting
    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let plane_scale = (player.fov * 0.5).tan();
    let plane_x = -dir_y * plane_scale;
    let plane_y =  dir_x * plane_scale;
    let pos_z = hh * 0.8;

    for i in 0..num_rays {
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        let intersect = cast_ray(framebuffer, maze, player, a, block_size, false);

        // Dirección del rayo para floor casting
        let camera_x = 2.0 * (i as f32) / (num_rays as f32) - 1.0;
        let ray_dir_x = dir_x + plane_x * camera_x;
        let ray_dir_y = dir_y + plane_y * camera_x;

        // === PAREDES CON TEXTURA ===
        let distance = intersect.distance.max(0.0001);
        let distance_to_projection_plane = 70.0;
        let stake_height = (hh / distance) * distance_to_projection_plane;

        let mut top = (hh - stake_height / 2.0).round() as i32;
        let mut bottom = (hh + stake_height / 2.0).round() as i32;
        let hmax = framebuffer.height as i32 - 1;
        if top < 0 { top = 0; }
        if bottom > hmax { bottom = hmax; }

        // Coordenada de textura horizontal (impacto)
        let hit_x = player.pos.x + distance * a.cos();
        let hit_y = player.pos.y + distance * a.sin();

        // ¿pared vertical u horizontal?
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
                    let (r, g, b, a_) = (wall_rgba[idx], wall_rgba[idx + 1], wall_rgba[idx + 2], wall_rgba[idx + 3]);
                    let shade = (1.0 / (1.0 + distance * 0.01)).min(1.0);
                    framebuffer.set_pixel_color(
                        i, y as u32,
                        Color::new((r as f32 * shade) as u8, (g as f32 * shade) as u8, (b as f32 * shade) as u8, a_)
                    );
                }
            }
        }

        // === SUELO CON TEXTURA ===
        for y in (bottom + 1)..screen_h {
            let p = (y as f32) - hh;
            if p.abs() < 0.1 { continue; }

            let row_dist = (pos_z / p).max(1.0);

            let world_x = player.pos.x + row_dist * ray_dir_x;
            let world_y = player.pos.y + row_dist * ray_dir_y;

            let texture_scale = 4.0;
            let texture_x = (world_x * texture_scale / block_size as f32).fract().abs();
            let texture_y = (world_y * texture_scale / block_size as f32).fract().abs();

            let u = ((texture_x * floor_w as f32) as usize).min(floor_w - 1);
            let v = ((texture_y * floor_h as f32) as usize).min(floor_h - 1);

            let idx = (v * floor_w + u) * 4;
            if idx + 3 < floor_rgba.len() {
                let (r, g, b, a_) = (floor_rgba[idx], floor_rgba[idx + 1], floor_rgba[idx + 2], floor_rgba[idx + 3]);
                let dim = (1.0 / (1.0 + row_dist * 0.001)).min(1.0);
                framebuffer.set_pixel_color(
                    i, y as u32,
                    Color::new((r as f32 * dim) as u8, (g as f32 * dim) as u8, (b as f32 * dim) as u8, a_)
                );
            }
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Example - Start / Win Screens")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(60);

    // Audio
    let audio = RaylibAudio::init_audio_device()
        .expect("No se pudo inicializar el dispositivo de audio");

    let footstep_sound: Sound = audio
        .new_sound("assets/footstep.wav")
        .expect("No se pudo cargar assets/footstep.wav");

    let mut music = audio
        .new_music("assets/background.ogg")
        .or_else(|_| audio.new_music("assets/background.mp3"))
        .expect("No se pudo cargar música (background.ogg/mp3)");
    music.play_stream();

    // Framebuffer y laberinto
    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::new(70, 130, 180, 255));

    let maze = load_maze("maze.txt");

    // Jugador
    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    // Texturas del mundo
    let (wall_rgba, wall_w, wall_h) = load_image_rgba("assets/wall.png");
    let (floor_rgba, floor_w, floor_h) = load_image_rgba("assets/floor.png");

    // ===== PANTALLAS: inicio y victoria =====
    let lobby_tex = window
        .load_texture(&raylib_thread, "assets/lobby.png")
        .expect("No se pudo cargar assets/lobby.png");
    let win_tex = window
        .load_texture(&raylib_thread, "assets/win.png")
        .ok(); // opcional

    enum GameState { Menu, Playing, Win }
    let mut state = GameState::Menu;

    let mut mode_2d = false;
    let mut last_footstep_time = std::time::Instant::now();

    while !window.window_should_close() {
        music.update_stream();

        match state {
            GameState::Menu => {
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    window.set_mouse_position(Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0));
                    state = GameState::Playing;
                    continue;
                }

                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);

                // Dibujar lobby.png centrado con aspect ratio
                let tw = lobby_tex.width() as f32;
                let th = lobby_tex.height() as f32;
                let sw = window_width as f32;
                let sh = window_height as f32;
                let scale = (sw / tw).min(sh / th);
                let dw = tw * scale;
                let dh = th * scale;
                let dx = (sw - dw) * 0.5;
                let dy = (sh - dh) * 0.5;

                d.draw_texture_pro(
                    &lobby_tex,
                    Rectangle::new(0.0, 0.0, tw, th),
                    Rectangle::new(dx, dy, dw, dh),
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );

                let txt = "Presiona ENTER para empezar";
                let size = 28;
                let wtxt = d.measure_text(txt, size);
                d.draw_text(txt, (window_width - wtxt) / 2, window_height - 60, size, Color::RAYWHITE);

                d.draw_fps(10, 10);
            }

            GameState::Playing => {
                // 1) limpiar framebuffer
                framebuffer.clear();

                // 2) Toggle 2D/3D
                if window.is_key_pressed(KeyboardKey::KEY_M) {
                    mode_2d = !mode_2d;
                }

                // 3) control con mouse (horizontal)
                let mouse_pos = window.get_mouse_position();
                let center_x = window_width as f32 / 2.0;
                let mouse_delta = mouse_pos.x - center_x;

                if mouse_delta.abs() > 1.0 {
                    let mouse_sensitivity = 0.003;
                    player.a += mouse_delta * mouse_sensitivity; // cambia a -= si quieres invertir
                    window.set_mouse_position(Vector2::new(center_x, mouse_pos.y));
                }

                // 4) mover jugador + pasos
                process_events_enhanced(
                    &mut player,
                    &window,
                    &maze,
                    block_size,
                    &footstep_sound,
                    &mut last_footstep_time,
                );

                // 5) si pisa 'g' => victoria
                let ci = (player.pos.x as usize) / block_size;
                let cj = (player.pos.y as usize) / block_size;
                if matches!(maze.get(cj).and_then(|r| r.get(ci)).copied(), Some('g')) {
                    state = GameState::Win;
                    continue;
                }

                // 6) dibujar mundo
                if mode_2d {
                    render_maze(&mut framebuffer, &maze, block_size, &player);
                } else {
                    render_world(
                        &mut framebuffer,
                        &maze,
                        block_size,
                        &player,
                        &wall_rgba, wall_w, wall_h,
                        &floor_rgba, floor_w, floor_h
                    );
                }

                // 7) presentar
                framebuffer.swap_buffers(&mut window, &raylib_thread);
            }

            GameState::Win => {
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    // Reinicia al menú (o cámbialo a Playing si prefieres)
                    state = GameState::Menu;
                    // Reseteo básico de jugador
                    player.pos = Vector2::new(150.0, 150.0);
                    player.a = PI / 3.0;
                    continue;
                }

                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);

                if let Some(ref tex) = win_tex {
                    let tw = tex.width() as f32;
                    let th = tex.height() as f32;
                    let sw = window_width as f32;
                    let sh = window_height as f32;
                    let scale = (sw / tw).min(sh / th);
                    let dw = tw * scale;
                    let dh = th * scale;
                    let dx = (sw - dw) * 0.5;
                    let dy = (sh - dh) * 0.5;

                    d.draw_texture_pro(
                        tex,
                        Rectangle::new(0.0, 0.0, tw, th),
                        Rectangle::new(dx, dy, dw, dh),
                        Vector2::new(0.0, 0.0),
                        0.0,
                        Color::WHITE,
                    );
                } else {
                    let title = "¡VICTORIA!";
                    let prompt = "Presiona ENTER para continuar";
                    let ts = 48;
                    let ps = 24;
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