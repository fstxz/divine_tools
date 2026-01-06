# Divine Tools

Tool for inspecting and modifying various binary files from Divine Divinity and Beyond Divinity.

## Usage

### Binary editor

Launching the program without any arguments will open the editor that allows to inspect and edit binary files.

### Packed `.cmp` files

Note: This only works with files that have other files embedded inside them. Full list of those files: `flat.cmp`, `global.cmp`, `sound.cmp`, `voice.cmp`.

To unpack a file, run `dt.exe unpack <path-to-cmp>` (or `dt` if you are on Linux), where `<path-to-cmp>` is the path to the `.cmp` file. Files will be extracted to the current working directory, so make sure it's empty to avoid overwriting other files.

To pack files in the current directory back into a `.cmp` file, run `dt.exe pack`. You can specify output file name with the `-o` argument, e.g. `dt.exe pack -o global.cmp`

## Installation

Prebuilt binaries are available in [releases](https://github.com/fstxz/divine_tools/releases) for Windows and Linux.

### Building from source

Install [rustup](https://rustup.rs/), then run the following commands to build the project:

```sh
git clone https://github.com/fstxz/divine_tools.git
cd divine_tools
cargo build --release
```

The `dt` binary will be placed in the `./target/release/` directory.

## License

This project is licensed under the [GPLv3](LICENSE.txt).
