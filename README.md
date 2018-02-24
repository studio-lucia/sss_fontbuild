# sss_fontbuild

`sss_fontbuild` is a commandline tool which creates font files that can be used with the Sega Saturn game *Lunar: Silver Star Story*. It was created for the English fan translation, and could also be used for other fan translations and similar projects.

## Usage

For more information on flags, run `sss_fontbuild --help`.

This tool supports two modes: creating a standalone font tileset, or inserting a font into an existing `SYSTEM.DAT` file.

### Creating a standalone tileset

This is the default mode. When called without any other options, the tool will create an uncompressed standalone tileset which can be viewed in a tile viewer. For example:

```
$ sss_fontbuild path_to_font_files font.bin
```

Options:

* `--compress`: Compresses the resulting file using Sega's CMP compression format. This is the format used in-game; the data needs to be compressed for it to be readable.
* `--append`: Append arbitrary extra data to the end of the font. This is useful since most non-Japanese language fonts use many fewer characters, leaving several kilobytes of unused space that can be repurposed for other content.

### Inserting a tileset into `SYSTEM.DAT`

`SYSTEM.DAT` is the location of the font in the original game. This mode inserts a usable font into the provided `SYSTEM.DAT` file, ready to be used in-game. The file will be modified in-place instead of creating a new file. For example:

```
$ sss_fontbuild path_to_font_files SYSTEM.DAT --insert
```

Options:

* `--append`: As described above.

## Installing

To install from source, just run `make install`; this will install the binary to `/usr/local`. You can also specify a different location to install to by specifying the `PREFIX` option, such as `make install PREFIX=/opt/local`.

Compiling the tool requires a [Rust](https://www.rust-lang.org) compiler with its Cargo package manager, and a C compiler.

## Creating a font

### Specifications

*Silver Star Story* uses a variable-width font; each character is a 16x16 tile with three displayable colours and one transparent colour. The game will automatically determine the width of your font for you as long as your letters are left-aligned within the 16x16 tile.

When creating a font, each character should be its own PNG image in a directory using numeric filenames. For example, if your font has 255 characters, you would name your images `000.png` through `255.png`. Your images should use 8-bit colour depth; if they are paletted with fewer colours or use a higher bit depth, they will be rejected. The supported colours are (in red, green, blue notation):

* 217, 217, 217
* 0, 16, 64
* 128, 128, 176

All other colours will be mapped to transparency.

### Mapping to text

The game has room for up to 459 characters (*Silver Star Story*) or 504 characters (*Silver Star Story Complete*) within its font. This is more than enough room for an ASCII font, which contains only 95 printable characters, and is also roomy enough for many other languages.

The font contains a few special characters, such as the symbol for the game's currency and magical symbols used as hints to a puzzle in Vane. You will probably want to map those to codepoints if you're translating to a non-English, non-Japanese language.

The game doesn't have a mapping between encodings and characters in the font. A codepoint in the game's text is a simple 16-bit index into the font. For my English translation, I've created an ASCII mapping by just making sure those indices coincide with ASCII codepoints; this does mean leaving a bunch of blank characters at the beginning of the font. For languages whose writing systems are small enough to fit within the limits, you'll probably want to use a similar trick.

## Contributing

1. Fork the repository
2. Create a new branch
3. Commit your changes
4. Open a pull request
5. 🎉

## Bugs and support

Please report any issues on this repository's issue tracker. I'll try to do whatever I can to help!

## Credits

This tool was written by Misty De Meo. It uses a Sega CMP compressor written by [@MrConan1](http://github.com/MrConan1).
