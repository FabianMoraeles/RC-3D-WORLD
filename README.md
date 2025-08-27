RC-3D-WORLD (Raycasting en Rust + Raylib)

Juego tipo raycaster (estilo Doom/Wolf3D) escrito en Rust.
Renderiza paredes y suelo con texturas, muestra un sprite animado sobre la meta (g), tiene música de fondo y sonido de pasos, y trae pantalla de inicio y pantalla de victoria.

Demo rápida

Menú: imagen assets/lobby.png. Presiona ENTER para empezar.

Juego: mueve al jugador por el laberinto hasta llegar a la celda g.

Victoria: se muestra assets/win.png. Presiona ENTER para volver al menú.

Controles

W / S o ↑ / ↓: avanzar / retroceder

A / D: strafe (lateral)

← / →: rotación fina

Mouse: rotación horizontal

M: alterna vista 2D (debug) / 3D

ENTER: aceptar en Menú y en Victoria

Requisitos

Rust (estable). Instala con rustup
.

Compilador C (para compilar raylib nativa):

Windows (MSVC): instala Visual Studio Build Tools (C++ tools).

Linux: build-essential, pkg-config, libasound2-dev, libx11-dev, libxrandr-dev, libxi-dev, libgl1-mesa-dev, etc.

macOS: xcode-select --install.

No necesitas instalar raylib manualmente: el crate raylib compila la lib por ti (asegúrate de tener toolchain C).

Estructura del proyecto
.
├─ Cargo.toml
├─ maze.txt
├─ src/
│  ├─ main.rs
│  ├─ caster.rs
│  ├─ framebuffer.rs
│  ├─ line.rs
│  ├─ maze.rs
│  └─ player.rs
└─ assets/
   ├─ wall.png
   ├─ floor.png
   ├─ lobby.png          # pantalla de inicio
   ├─ win.png            # pantalla de victoria
   ├─ background.ogg     # música (ogg/mp3/ wav, uno es suficiente)
   ├─ footstep.wav       # pasos
   ├─ freddy1.png        # sprite frame 1 (sobre la 'g')
   └─ freddy2.png        # sprite frame 2 (sobre la 'g')


Si no tienes la carpeta assets/, créala en la raíz del repo y coloca ahí todas las imágenes y audio.

maze.txt de ejemplo

El proyecto ya funciona con este laberinto (la g es la meta):

+--+--+--+--+
|           |
+  +--+  +  +
|  |     |  |
+  +  +--+--+
|  |        |
+  +--+--+  +
|        | g|
+--+--+--+--+

Configuración en Cargo.toml

Ejemplo mínimo (ajústalo si ya lo tienes):

[package]
name = "computer-graphics-v3"
version = "0.1.0"
edition = "2021"

[profile.dev]
opt-level = 3
debug = false

[dependencies]
raylib = "5.5.1"
image = { version = "0.25.6", default-features = false, features = ["png"] }


Nota: la API de audio se usa vía RaylibAudio (no requiere feature especial en el crate raylib 5.5.x).

Cómo correr
# desde la carpeta del proyecto
cargo run
