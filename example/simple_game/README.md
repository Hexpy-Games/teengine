# Simple Game Example

This is a simple 2D game example built with the Teengine game engine. It demonstrates basic sprite animation, character movement, and game engine features.

## Features

- 2D sprite rendering with animation support
- Keyboard input handling
- Sprite animation state management
- Color key transparency for sprites

## Prerequisites

- Rust (Latest stable version)
- OpenGL compatible graphics driver
- Required dependencies (listed in Cargo.toml)

## Building and Running

1. Make sure you have Rust installed on your system
2. Clone the repository
3. Navigate to the simple_game directory
4. Run the game:

```bash
cargo run --release
```

## Controls

- Arrow keys or WASD: Move the character
- ESC: Exit game

## Project Structure

```
simple_game/
├── src/
│   └── main.rs       # Main game implementation
├── assets/
│   └── sprite.png    # Sprite sheet for character
├── Cargo.toml        # Project dependencies
└── README.md         # This file
```

## Dependencies

- teengine = { path = "../.." }
- glam = "0.24.1"

## Contributing

Feel free to submit issues and enhancement requests!

## License

This example game is released under the MIT License. See the LICENSE file for details.