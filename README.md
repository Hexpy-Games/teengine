# Teengine
![Rust Version](https://img.shields.io/badge/rust-1.80.1%2B-orange.svg)
[![License](https://img.shields.io/badge/License-Apache%202.0%20with%20Commons%20Clause-blue.svg)](LICENSE)

Teengine is a 2D/3D game engine developed using the Rust programming language. This project is currently focused on studying game engine development and is not yet ready for production use.

## Current Features
- Cross-platform window creation and event handling
- 2D sprite rendering with animation support
- Basic OpenGL-based rendering system
- Input handling system (keyboard support)
- Color key transparency for sprites
- Simple game state management

## Planned Features
- Extended input handling (mouse, gamepad)
- Physics engine integration (2D/3D)
- Resource management system
- Audio system
- 3D rendering capabilities
- UI system
- Scene management

## Getting Started

### Requirements
- Rust 1.80 or higher
- Cargo (Rust package manager)
- Graphics card and driver supporting OpenGL 3.3 or higher

### Installation
1. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/teengine.git
   cd teengine
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the example game:
   ```bash
   cd examples/simple_game
   cargo run --release
   ```

## Usage
Check out the `examples/simple_game` directory for a basic implementation example. This example demonstrates:
- Basic window creation
- Sprite rendering
- Animation system
- Input handling
- Game state management

For more detailed usage instructions, please refer to the [documentation](docs/USAGE.md).

## Project Structure
```
teengine/
├── src/             # Engine core implementation
├── examples/        # Example projects
│   └── simple_game/ # Basic game implementation
├── docs/           # Documentation
└── tests/          # Engine tests
```

## Contributing
We welcome contributions! If you'd like to contribute to the project, please refer to [CONTRIBUTING.md](CONTRIBUTING.md).

## License
This project is licensed under the Apache License, Version 2.0 with Commons Clause.
See the [LICENSE](LICENSE) file for the complete license terms.

## Acknowledgments
Thanks to all contributors and the Rust gamedev community for their support and inspiration.