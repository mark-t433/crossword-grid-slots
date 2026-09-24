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
an open cell. Use `.` for an empty grid, or put real letters in if you
already have a filled one — each slot's letters are echoed back in its
output.

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

It also reads from stdin, either when no file argument is given or when
the argument is `-`, so it fits in a pipeline:

```
cat example.txt | cargo run
```

Output:

```
1A row=0 col=0 len=3 text=...
1D row=0 col=0 len=3 text=...
2D row=0 col=2 len=3 text=...
3A row=2 col=0 len=3 text=...
```

Reading that: slot 1 starts at row 0, col 0 and runs both across and
down for 3 cells. Slot 2 is a down-only entry starting at row 0, col 2.
Slot 3 is an across-only entry starting at row 2, col 0. The center
cell is blocked, so it never gets a number and neither the middle row
nor the middle column runs through it. `text` is `.` for every cell
here because `example.txt` is unfilled; a grid with real letters would
show them uppercased, e.g. `text=CAT`, with `.` only where that slot
crosses a still-blank cell.

Pass `--format json` for machine-readable output instead:

```
cargo run -- --format json example.txt
```

```
[{"number":1,"direction":"across","row":0,"col":0,"length":3,"text":"..."},{"number":1,"direction":"down","row":0,"col":0,"length":3,"text":"..."},{"number":2,"direction":"down","row":0,"col":2,"length":3,"text":"..."},{"number":3,"direction":"across","row":2,"col":0,"length":3,"text":"..."}]
```

The default is `--format text`. `--format=json` also works.

## Library

The CLI is a thin wrapper around three functions in `src/lib.rs`:

```rust
let grid = gridslots::parse_grid(&text)?;
let slots = gridslots::slots(&grid);
let json = gridslots::to_json(&slots);
```

All three are pure: `parse_grid` turns text into a `Grid`, `slots` turns
a `Grid` into a `Vec<Slot>`, and `to_json` turns a slice of `Slot` into
a JSON string. None of them touch the filesystem or any other state.
That makes every case, including edge cases like a single open cell or
a fully blocked grid, a plain input-output test — see the tests at the
bottom of `src/lib.rs`.

## License

MIT, see `LICENSE`.
