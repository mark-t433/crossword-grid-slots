//! Crossword grid slot numbering.
//!
//! A crossword grid is just a rectangle of open cells and black squares.
//! The shape of that rectangle alone determines where every across and
//! down entry starts, how long it is, and what number it gets under the
//! usual convention (scan left-to-right, top-to-bottom; a cell starts a
//! new numbered entry if it begins an across word, a down word, or both).
//!
//! This crate answers exactly that question and nothing else: given a
//! grid shape, what are its numbered slots?

/// A parsed grid shape: dimensions plus the character in each cell.
/// `#` means a black square; anything else is an open cell, which may
/// carry a letter (if the source text had a fill) or just a placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    rows: usize,
    cols: usize,
    cells: Vec<char>,
}

impl Grid {
    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn is_block(&self, row: usize, col: usize) -> bool {
        self.cells[row * self.cols + col] == '#'
    }

    /// The letter filled into an open cell, if the source text had one.
    /// `None` for black squares and for open cells that are still blank
    /// (any non-alphabetic placeholder, typically `.`).
    pub fn letter(&self, row: usize, col: usize) -> Option<char> {
        let ch = self.cells[row * self.cols + col];
        ch.is_alphabetic().then(|| ch.to_ascii_uppercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Across,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub number: u32,
    pub direction: Direction,
    pub row: usize,
    pub col: usize,
    pub length: usize,
    /// One character per cell in the slot, in reading order: the filled
    /// letter where the grid has one, `.` where it's still blank.
    pub text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyGrid,
    RaggedRow {
        row: usize,
        expected: usize,
        found: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyGrid => write!(f, "grid text is empty"),
            ParseError::RaggedRow {
                row,
                expected,
                found,
            } => write!(f, "row {row} has {found} columns, expected {expected}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a grid from plain text: one line per row, `#` for a black
/// square, any other character for an open cell. Blank lines are
/// ignored so a trailing newline in a file doesn't matter.
pub fn parse_grid(text: &str) -> Result<Grid, ParseError> {
    let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();

    let cols = match lines.first() {
        Some(first) => first.chars().count(),
        None => return Err(ParseError::EmptyGrid),
    };

    let mut cells = Vec::with_capacity(lines.len() * cols);
    for (row, line) in lines.iter().enumerate() {
        let found = line.chars().count();
        if found != cols {
            return Err(ParseError::RaggedRow {
                row,
                expected: cols,
                found,
            });
        }
        cells.extend(line.chars());
    }

    Ok(Grid {
        rows: lines.len(),
        cols,
        cells,
    })
}

/// Returns every numbered slot in the grid, in the order a solver would
/// number them: row by row, left to right, across before down when a
/// cell starts both.
pub fn slots(grid: &Grid) -> Vec<Slot> {
    let mut result = Vec::new();
    let mut next_number = 1;

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if grid.is_block(row, col) {
                continue;
            }

            let starts_across = starts_across(grid, row, col);
            let starts_down = starts_down(grid, row, col);

            if !starts_across && !starts_down {
                continue;
            }

            let number = next_number;
            next_number += 1;

            if starts_across {
                let length = across_length(grid, row, col);
                result.push(Slot {
                    number,
                    direction: Direction::Across,
                    row,
                    col,
                    length,
                    text: slot_text(grid, Direction::Across, row, col, length),
                });
            }
            if starts_down {
                let length = down_length(grid, row, col);
                result.push(Slot {
                    number,
                    direction: Direction::Down,
                    row,
                    col,
                    length,
                    text: slot_text(grid, Direction::Down, row, col, length),
                });
            }
        }
    }

    result
}

/// Black squares whose 180-degree rotational partner is open, in reading
/// order. Most newspaper grids are required to be symmetric this way, but
/// plenty of valid grids aren't, so this only reports; it never rejects.
/// An empty result means the grid is symmetric. Each mismatch is reported
/// once, on the block side, which is the square a constructor would need
/// to add or remove a partner for.
pub fn asymmetric_blocks(grid: &Grid) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if grid.is_block(row, col)
                && !grid.is_block(grid.rows - 1 - row, grid.cols - 1 - col)
            {
                result.push((row, col));
            }
        }
    }
    result
}

fn starts_across(grid: &Grid, row: usize, col: usize) -> bool {
    let left_is_boundary = col == 0 || grid.is_block(row, col - 1);
    let right_is_open = col + 1 < grid.cols && !grid.is_block(row, col + 1);
    left_is_boundary && right_is_open
}

fn starts_down(grid: &Grid, row: usize, col: usize) -> bool {
    let above_is_boundary = row == 0 || grid.is_block(row - 1, col);
    let below_is_open = row + 1 < grid.rows && !grid.is_block(row + 1, col);
    above_is_boundary && below_is_open
}

impl Direction {
    fn as_str(&self) -> &'static str {
        match self {
            Direction::Across => "across",
            Direction::Down => "down",
        }
    }
}

/// Renders slots as a JSON array of objects, in the same order `slots`
/// produced them. No external crate needed: every field is a plain
/// number, a fixed enum string, or `text`, which only ever holds
/// uppercase letters and `.`, so nothing here needs escaping.
pub fn to_json(slots: &[Slot]) -> String {
    let mut out = String::from("[");
    for (i, slot) in slots.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"number\":{number},\"direction\":\"{direction}\",\"row\":{row},\"col\":{col},\"length\":{length},\"text\":\"{text}\"}}",
            number = slot.number,
            direction = slot.direction.as_str(),
            row = slot.row,
            col = slot.col,
            length = slot.length,
            text = slot.text,
        ));
    }
    out.push(']');
    out
}

fn slot_text(grid: &Grid, direction: Direction, row: usize, col: usize, length: usize) -> String {
    (0..length)
        .map(|i| {
            let (r, c) = match direction {
                Direction::Across => (row, col + i),
                Direction::Down => (row + i, col),
            };
            grid.letter(r, c).unwrap_or('.')
        })
        .collect()
}

fn across_length(grid: &Grid, row: usize, col: usize) -> usize {
    let mut len = 0;
    let mut c = col;
    while c < grid.cols && !grid.is_block(row, c) {
        len += 1;
        c += 1;
    }
    len
}

fn down_length(grid: &Grid, row: usize, col: usize) -> usize {
    let mut len = 0;
    let mut r = row;
    while r < grid.rows && !grid.is_block(r, col) {
        len += 1;
        r += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_an_error() {
        assert_eq!(parse_grid(""), Err(ParseError::EmptyGrid));
    }

    #[test]
    fn ragged_rows_are_rejected() {
        let err = parse_grid("...\n..\n...").unwrap_err();
        assert_eq!(
            err,
            ParseError::RaggedRow {
                row: 1,
                expected: 3,
                found: 2,
            }
        );
    }

    #[test]
    fn single_cell_grid_has_no_slots() {
        let grid = parse_grid(".").unwrap();
        assert_eq!(slots(&grid), Vec::new());
    }

    #[test]
    fn all_black_grid_has_no_slots() {
        let grid = parse_grid("##\n##").unwrap();
        assert_eq!(slots(&grid), Vec::new());
    }

    #[test]
    fn open_3x3_grid_numbers_every_row_and_column() {
        let grid = parse_grid("...\n...\n...").unwrap();
        let result = slots(&grid);

        let expected = vec![
            Slot { number: 1, direction: Direction::Across, row: 0, col: 0, length: 3, text: "...".into() },
            Slot { number: 1, direction: Direction::Down, row: 0, col: 0, length: 3, text: "...".into() },
            Slot { number: 2, direction: Direction::Down, row: 0, col: 1, length: 3, text: "...".into() },
            Slot { number: 3, direction: Direction::Down, row: 0, col: 2, length: 3, text: "...".into() },
            Slot { number: 4, direction: Direction::Across, row: 1, col: 0, length: 3, text: "...".into() },
            Slot { number: 5, direction: Direction::Across, row: 2, col: 0, length: 3, text: "...".into() },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn center_block_splits_the_middle_row_and_column() {
        let grid = parse_grid("...\n.#.\n...").unwrap();
        let result = slots(&grid);

        let expected = vec![
            Slot { number: 1, direction: Direction::Across, row: 0, col: 0, length: 3, text: "...".into() },
            Slot { number: 1, direction: Direction::Down, row: 0, col: 0, length: 3, text: "...".into() },
            Slot { number: 2, direction: Direction::Down, row: 0, col: 2, length: 3, text: "...".into() },
            Slot { number: 3, direction: Direction::Across, row: 2, col: 0, length: 3, text: "...".into() },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn is_block_reflects_source_text() {
        let grid = parse_grid("#.\n..").unwrap();
        assert!(grid.is_block(0, 0));
        assert!(!grid.is_block(0, 1));
        assert!(!grid.is_block(1, 0));
        assert!(!grid.is_block(1, 1));
    }

    #[test]
    fn symmetric_grid_has_no_asymmetric_blocks() {
        let grid = parse_grid("#..\n...\n..#").unwrap();
        assert_eq!(asymmetric_blocks(&grid), Vec::new());
    }

    #[test]
    fn center_block_is_its_own_partner() {
        let grid = parse_grid("...\n.#.\n...").unwrap();
        assert_eq!(asymmetric_blocks(&grid), Vec::new());
    }

    #[test]
    fn lone_corner_block_is_reported() {
        let grid = parse_grid("#..\n...\n...").unwrap();
        assert_eq!(asymmetric_blocks(&grid), vec![(0, 0)]);
    }

    #[test]
    fn mirrored_across_an_axis_is_not_rotational_symmetry() {
        // Blocks at both top corners are a mirror image, not a rotation.
        let grid = parse_grid("#.#\n...\n...").unwrap();
        assert_eq!(asymmetric_blocks(&grid), vec![(0, 0), (0, 2)]);
    }

    #[test]
    fn to_json_renders_empty_slots_as_empty_array() {
        assert_eq!(to_json(&[]), "[]");
    }

    #[test]
    fn to_json_renders_one_slot_per_object() {
        let grid = parse_grid("...\n.#.\n...").unwrap();
        let result = slots(&grid);

        assert_eq!(
            to_json(&result),
            "[\
             {\"number\":1,\"direction\":\"across\",\"row\":0,\"col\":0,\"length\":3,\"text\":\"...\"},\
             {\"number\":1,\"direction\":\"down\",\"row\":0,\"col\":0,\"length\":3,\"text\":\"...\"},\
             {\"number\":2,\"direction\":\"down\",\"row\":0,\"col\":2,\"length\":3,\"text\":\"...\"},\
             {\"number\":3,\"direction\":\"across\",\"row\":2,\"col\":0,\"length\":3,\"text\":\"...\"}\
             ]"
        );
    }

    #[test]
    fn letters_are_echoed_back_uppercased_per_slot() {
        let grid = parse_grid("cat\n.#.\n...").unwrap();
        let result = slots(&grid);

        assert_eq!(result[0].text, "CAT"); // 1 across
        assert_eq!(result[1].text, "C.."); // 1 down, only the shared cell is filled
    }

    #[test]
    fn blank_placeholder_characters_are_not_treated_as_letters() {
        let grid = parse_grid("_-_\n___").unwrap();
        assert_eq!(grid.letter(0, 0), None);
        assert_eq!(slots(&grid)[0].text, "...");
    }
}
