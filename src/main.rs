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
use player::{Player, process_events};

use raylib::prelude::*;
use std::f32::consts::PI;

// ===== Helpers de visualización 2D =====

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
    let color = cell_to_color(cell);
    framebuffer.set_current_color(color);

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
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }

    // Rayos de depuración (vista 2D)
    framebuffer.set_current_color(Color::WHITESMOKE);
    let num_rays = 5;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        // draw_line = true para visualizar
        let _ = cast_ray(framebuffer, maze, player, a, block_size, true);
    }
}

// ===== Render 3D =====

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
) {
    let num_rays = framebuffer.width;        // una columna por píxel de ancho
    let hh = framebuffer.height as f32 / 2.0;

    framebuffer.set_current_color(Color::WHITESMOKE);

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, maze, player, a, block_size, false);

        // distancia saneada para evitar división por 0
        let distance = intersect.distance.max(0.0001);

        // proyección simple: altura ∝ 1 / distancia
        let distance_to_projection_plane = 70.0;
        let stake_height = (hh / distance) * distance_to_projection_plane;

        // clamp a pantalla
        let mut top = (hh - stake_height / 2.0).round() as i32;
        let mut bottom = (hh + stake_height / 2.0).round() as i32;
        let hmax = framebuffer.height as i32 - 1;
        if top < 0 { top = 0; }
        if bottom > hmax { bottom = hmax; }

        if bottom >= top {
            for y in top..=bottom {
                framebuffer.set_pixel(i, y as u32);
            }
        }
    }
}

// ===== main =====

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Example")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));

    let maze = load_maze("maze.txt");

    // Asegúrate que esta celda en maze.txt sea ' ' (pasillo)
    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    // Toggle de modo 2D/3D persistente
    let mut mode_2d = false;

    while !window.window_should_close() {
        // 1) limpiar framebuffer
        framebuffer.clear();

        // 2) toggle de modo (solo una vez por pulsación)
        if window.is_key_pressed(KeyboardKey::KEY_M) {
            mode_2d = !mode_2d;
        }

        // 3) mover jugador con colisión (player.rs actualizado)
        process_events(&mut player, &window, &maze, block_size);

        // 4) dibujar
        if mode_2d {
            render_maze(&mut framebuffer, &maze, block_size, &player);
        } else {
            render_world(&mut framebuffer, &maze, block_size, &player);
        }

        // 5) presentar
        framebuffer.swap_buffers(&mut window, &raylib_thread);

        // ~60 FPS
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
