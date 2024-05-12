# Rusty FreeCell

A [FreeCell](https://en.wikipedia.org/wiki/FreeCell) solitaire card game in Rust, with a text-based user interface.

## How To Play

Head over to the [releases page](releases/) and grab a binary for Windows, MacOS or Linux. It's a command-line application, so run it from your terminal.

```
./rustyfreecell
```

For FreeCell solitaire rules, [check Wikipedia](https://en.wikipedia.org/wiki/FreeCell).

### Controls

<kbd>←</kbd> (or <kbd>A</kbd>) - Move cursor left

<kbd>→</kbd> (or <kbd>D</kbd>) - Move cursor right

<kbd>SPACE</kbd> (or <kbd>ENTER</kbd>) - Select/move card

<kbd>Z</kbd> - Undo (step back in history)

<kbd>F</kbd> - Quick stack all visible cards to foundation (recursive)

<kbd>H</kbd> - Toggle high contrast display mode

<kbd>CTRL</kbd> + <kbd>N</kbd> - New Game

<kbd>CTRL</kbd> + <kbd>Q</kbd> - Quit to terminal

## Building From Source

To build and run Rusty FreeCell from source, [install Rust using rustup](https://www.rust-lang.org/tools/install). Then in the source directory:

```
cargo run --release
```
