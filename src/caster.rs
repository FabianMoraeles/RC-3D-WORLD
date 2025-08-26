use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
  pub distance: f32,
  pub impact: char,
}

pub fn cast_ray(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  a: f32,
  block_size: usize,
  draw_line: bool,
) -> Intersect {
  debug_assert!(block_size > 0, "block_size debe ser > 0");

  // Paso pequeño para no saltarnos paredes finas
  let step: f32 = 2.0;          // px
  let max_dist: f32 = 20000.0;  // tope anti-bucle

  let cos_a = a.cos();
  let sin_a = a.sin();

  let mut d = 0.0;
  framebuffer.set_current_color(Color::WHITESMOKE);

  loop {
    let fx = player.pos.x + d * cos_a;
    let fy = player.pos.y + d * sin_a;

    // Si salimos del mapa por negativo, considerar pared exterior
    if fx < 0.0 || fy < 0.0 {
      return Intersect { distance: d.max(0.0001), impact: '#' };
    }

    let x = fx as usize;
    let y = fy as usize;

    let i = x / block_size;
    let j = y / block_size;

    // Acceso SEGURO al mapa
    if let Some(&tile) = maze.get(j).and_then(|row| row.get(i)) {
      if tile != ' ' {
        // evita 0.0 para no dividir entre cero arriba
        return Intersect { distance: d.max(0.0001), impact: tile };
      }
    } else {
      // Fuera del mapa: pared exterior
      return Intersect { distance: d.max(0.0001), impact: '#' };
    }

    if draw_line {
      framebuffer.set_pixel(x as u32, y as u32);
    }

    d += step;
    if d > max_dist {
      return Intersect { distance: max_dist, impact: ' ' };
    }
  }
}
