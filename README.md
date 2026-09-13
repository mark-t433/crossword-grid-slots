# gridslots

Given the shape of a crossword grid, where does every across and down
entry start, how long is it, and what number does it get?

That numbering is entirely mechanical once you know which squares are
black: scan the grid left to right, top to bottom, and any open cell
that begins an across word, a down word, or both, gets the next number.
Constructors need this to lay out a grid before filling it in, and
solvers' apps need it to render clue numbers next to the right squares.
Working it out by hand for anything bigger than a 5x5 is tedious and
easy to get wrong at the edges. This tool does the arithmetic.

It reads a plain-text grid and prints every numbered slot. Nothing
else — no word fill, no clue database, no symmetry checking. Just the
one question: what are the slots?

## Grid format

One line per row. `#` marks a black square; any other character marks
an open cell (so you can use `.` for an empty grid or real letters if
you already have a filled one — letters are not currently echoed back,
only the shape matters).

`example.txt`:

```
...
.#.
...
```

## Usage

```
cargo run -- example.txt
```

Output:

```
1A row=0 col=0 len=3
1D row=0 col=0 len=3
2D row=0 col=2 len=3
3A row=2 col=0 len=3
```

Reading that: slot 1 starts at row 0, col 0 and runs both across and
down for 3 cells. Slot 2 is a down-only entry starting at row 0, col 2.
Slot 3 is an across-only entry starting at row 2, col 0. The center
cell is blocked, so it never gets a number and neither the middle row
nor the middle column runs through it.

## Library

The CLI is a thin wrapper around two functions in `src/lib.rs`:

```rust
let grid = gridslots::parse_grid(&text)?;
let slots = gridslots::slots(&grid);
```

Both are pure: `parse_grid` turns text into a `Grid`, `slots` turns a
`Grid` into a `Vec<Slot>`, and neither touches the filesystem or any
other state. That makes every case, including edge cases like a single
open cell or a fully blocked grid, a plain input-output test — see the
tests at the bottom of `src/lib.rs`.

## License

MIT, see `LICENSE`.
