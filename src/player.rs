use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
}

fn can_move(to_x: f32, to_y: f32, maze: &Maze, block_size: usize) -> bool {
    if to_x < 0.0 || to_y < 0.0 { return false; }
    let i = (to_x as usize) / block_size;
    let j = (to_y as usize) / block_size;
    if let Some(row) = maze.get(j) {
        if let Some(&cell) = row.get(i) {
            return cell == ' ';
        }
    }
    false
}

pub fn process_events(
    player: &mut Player,
    rl: &RaylibHandle,
    maze: &Maze,
    block_size: usize,
) {
    const MOVE_SPEED: f32 = 10.0;
    const ROT_SPEED: f32 = PI / 50.0;

    // rotación
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a += ROT_SPEED; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a -= ROT_SPEED; }

    let dir_x = player.a.cos();
    let dir_y = player.a.sin();

    // atrás
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        let nx = player.pos.x - MOVE_SPEED * dir_x;
        let ny = player.pos.y - MOVE_SPEED * dir_y;
        if can_move(nx, player.pos.y, maze, block_size) { player.pos.x = nx; }
        if can_move(player.pos.x, ny, maze, block_size) { player.pos.y = ny; }
    }
    // adelante
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        let nx = player.pos.x + MOVE_SPEED * dir_x;
        let ny = player.pos.y + MOVE_SPEED * dir_y;
        if can_move(nx, player.pos.y, maze, block_size) { player.pos.x = nx; }
        if can_move(player.pos.x, ny, maze, block_size) { player.pos.y = ny; }
    }
}
