# Key Remapper

Key Remapper is a powerful and flexible tool for remapping keyboard keys on Linux systems. It allows you to customize your keyboard layout, create complex key mappings, and enhance your typing experience.

## Features

- Remap any key to another key
- Configure different actions for tap and hold events
- Easy-to-use TOML configuration file
- Support for a wide range of keyboard keys
- Minimal overhead and efficient performance

## Prerequisites

Before you begin, ensure you have the following installed on your Linux system:

- Rust programming language (latest stable version)
- Cargo package manager
- `libudev` development files (usually `libudev-dev` package)

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/key-remapper.git
   cd key-remapper
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. The compiled binary will be available at `target/release/key-remapper`.

## Configuration

Key Remapper uses a TOML configuration file to define key mappings. Create a file named `key_remapper_config.toml` in the same directory as the executable.

Here's an example configuration:

```toml
[[key_mappings]]
key = "KEY_CAPSLOCK"
on_tap = "KEY_ESC"
on_hold = "KEY_LEFTMETA"

[[key_mappings]]
key = "KEY_RIGHTALT"
on_tap = "KEY_RIGHTALT"
on_hold = "KEY_RIGHTCTRL"
```

In this example:
- CapsLock is remapped to act as Escape when tapped and as the Left Meta key (usually Windows key) when held.
- Right Alt is remapped to act as Right Alt when tapped and as Right Ctrl when held.

For a full list of available key names, refer to the `evdev-key-names.md` file in the repository.

## Usage

1. Ensure your configuration file (`key_remapper_config.toml`) is in the same directory as the Key Remapper executable.

2. Run Key Remapper with root privileges:
   ```
   sudo ./key-remapper
   ```

3. The program will display a list of available input devices. Enter the number corresponding to the keyboard you want to remap.

4. Key Remapper will start running and apply your configured mappings.

5. To stop Key Remapper, use Ctrl+C.

## Troubleshooting

- If you encounter permission issues, make sure you're running the program with sudo.
- If your key mappings aren't working as expected, double-check your `key_remapper_config.toml` file for any syntax errors.
- Ensure that the key names in your configuration file match exactly with those listed in `evdev-key-names.md`.

## Contributing

Contributions to Key Remapper are welcome! Please feel free to submit pull requests, create issues, or suggest new features.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- This project uses the `evdev` crate for handling input events.
- Thanks to all contributors and users of Key Remapper.

## Disclaimer

Be cautious when remapping keys, especially system keys. Always ensure you have a way to undo your changes or access your system if something goes wrong. The authors are not responsible for any issues arising from the use of this software.
