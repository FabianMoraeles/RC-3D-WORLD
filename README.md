# RC-3D-WORLD (Raycasting en Rust + Raylib)

Proyecto tipo **raycaster** (estilo Wolf3D/Doom clásico) hecho en **Rust** con **raylib**.  
Incluye texturas para paredes y piso, **menú de inicio**, **pantalla de victoria**, **música de fondo**, **sonido de pasos** y un **sprite animado** sobre la meta (`g`).

## Gameplay

- **Menú**: se muestra `assets/lobby.png`. Presiona **ENTER** para comenzar.
- **Objetivo**: llega a la celda **`g`** del laberinto (definido en `maze.txt`).
- **Victoria**: al tocar `g` se muestra `assets/win.png`. Desde allí, **ENTER** vuelve al menú.
- **Sprite animado**: sobre la `g` se dibuja un sprite que alterna entre `freddy1.png` y `freddy2.png`.

## Controles

- **W / S** o **↑ / ↓**: avanzar / retroceder  
- **A / D**: desplazamiento lateral (strafe)  
- **← / →**: rotación fina  
- **Mouse**: rotación horizontal  
- **M**: alterna vista **2D** (debug) / **3D**  
- **ENTER**: aceptar en Menú y en Victoria

---

## Requisitos

- **Rust** (estable). Instala con [rustup.rs](https://rustup.rs/).
- **Toolchain C** (para compilar raylib nativa):
  - **Windows (MSVC)**: instalar *Visual Studio Build Tools* → C++ build tools.
  - **Linux**: `build-essential`, `pkg-config`, `libasound2-dev`, `libx11-dev`, `libxrandr-dev`, `libxi-dev`, `libgl1-mesa-dev`, etc.
  - **macOS**: `xcode-select --install`.

> El crate `raylib` compila/trae la lib por ti, pero necesita las herramientas del sistema.

## Estructura del proyecto

.
├─ Cargo.toml
├─ maze.txt
├─ src/
│ ├─ main.rs
│ ├─ caster.rs
│ ├─ framebuffer.rs
│ ├─ line.rs
│ ├─ maze.rs
│ └─ player.rs
└─ assets/
├─ wall.png
├─ floor.png
├─ lobby.png # pantalla de inicio
├─ win.png # pantalla de victoria
├─ background.ogg # música (puede ser .mp3 también)
├─ footstep.wav # sonido de pasos
├─ freddy1.png # sprite frame 1 (sobre la 'g')
└─ freddy2.png # sprite frame 2 (sobre la 'g')

markdown
Copiar código

> Asegúrate de **crear la carpeta `assets/`** y colocar todos los archivos indicados.

## `maze.txt` de ejemplo

+--+--+--+--+
| |

+--+ + +
| | | |

+--+--+
| | |

+--+--+ +
| | g|
+--+--+--+--+

pgsql
Copiar código

## `Cargo.toml` (ejemplo)

> Usa tu edición preferida (2021/2024). Ajusta si ya lo tienes uno.

```toml
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
Cómo ejecutar
bash
Copiar código
# en la raíz del proyecto
cargo run
Para más rendimiento:

bash
Copiar código
cargo run --release

Cómo correr
# desde la carpeta del proyecto
cargo run
