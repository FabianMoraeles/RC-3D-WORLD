use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
  pub distance: f32,
  pub impact: char
}

pub fn cast_ray(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  a: f32,
  block_size: usize,
  draw_line: bool,
) -> Intersect {
  let mut d = 0.0;

  framebuffer.set_current_color(Color::WHITESMOKE);

  loop {
    let cos = d * a.cos();
    let sin = d * a.sin();
    let x = (player.pos.x + cos) as isize;
    let y = (player.pos.y + sin) as isize;

    // Si salimos de la pantalla, termina el rayo
    if x < 0 || y < 0 || (x as u32) >= framebuffer.width || (y as u32) >= framebuffer.height {
      return Intersect { distance: d.max(0.001), impact: ' ' };
    }

    let i = (x as usize) / block_size;
    let j = (y as usize) / block_size;

    // Bounds del maze
    if j >= maze.len() || i >= maze[j].len() {
      return Intersect { distance: d.max(0.001), impact: ' ' };
    }

    let cell = maze[j][i];

    // Impacta en cualquier cosa que no sea ' ' NI 'g' (la meta no es pared)
    if cell != ' ' && cell != 'g' {
      return Intersect { distance: d.max(0.001), impact: cell };
    }

    if draw_line {
      framebuffer.set_pixel(x as u32, y as u32);
    }

    d += 5.0; // paso más fino para precisión
  }
}
