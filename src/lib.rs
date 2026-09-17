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

/// A parsed grid shape: dimensions plus which cells are black squares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    rows: usize,
    cols: usize,
    blocks: Vec<bool>,
}

impl Grid {
    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn is_block(&self, row: usize, col: usize) -> bool {
        self.blocks[row * self.cols + col]
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

    let mut blocks = Vec::with_capacity(lines.len() * cols);
    for (row, line) in lines.iter().enumerate() {
        let found = line.chars().count();
        if found != cols {
            return Err(ParseError::RaggedRow {
                row,
                expected: cols,
                found,
            });
        }
        blocks.extend(line.chars().map(|ch| ch == '#'));
    }

    Ok(Grid {
        rows: lines.len(),
        cols,
        blocks,
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
                result.push(Slot {
                    number,
                    direction: Direction::Across,
                    row,
                    col,
                    length: across_length(grid, row, col),
                });
            }
            if starts_down {
                result.push(Slot {
                    number,
                    direction: Direction::Down,
                    row,
                    col,
                    length: down_length(grid, row, col),
                });
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
/// produced them. No external crate needed: the fields are all plain
/// numbers or fixed enum strings, so there's nothing to escape.
pub fn to_json(slots: &[Slot]) -> String {
    let mut out = String::from("[");
    for (i, slot) in slots.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"number\":{number},\"direction\":\"{direction}\",\"row\":{row},\"col\":{col},\"length\":{length}}}",
            number = slot.number,
            direction = slot.direction.as_str(),
            row = slot.row,
            col = slot.col,
            length = slot.length,
        ));
    }
    out.push(']');
    out
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
            Slot { number: 1, direction: Direction::Across, row: 0, col: 0, length: 3 },
            Slot { number: 1, direction: Direction::Down, row: 0, col: 0, length: 3 },
            Slot { number: 2, direction: Direction::Down, row: 0, col: 1, length: 3 },
            Slot { number: 3, direction: Direction::Down, row: 0, col: 2, length: 3 },
            Slot { number: 4, direction: Direction::Across, row: 1, col: 0, length: 3 },
            Slot { number: 5, direction: Direction::Across, row: 2, col: 0, length: 3 },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn center_block_splits_the_middle_row_and_column() {
        let grid = parse_grid("...\n.#.\n...").unwrap();
        let result = slots(&grid);

        let expected = vec![
            Slot { number: 1, direction: Direction::Across, row: 0, col: 0, length: 3 },
            Slot { number: 1, direction: Direction::Down, row: 0, col: 0, length: 3 },
            Slot { number: 2, direction: Direction::Down, row: 0, col: 2, length: 3 },
            Slot { number: 3, direction: Direction::Across, row: 2, col: 0, length: 3 },
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
             {\"number\":1,\"direction\":\"across\",\"row\":0,\"col\":0,\"length\":3},\
             {\"number\":1,\"direction\":\"down\",\"row\":0,\"col\":0,\"length\":3},\
             {\"number\":2,\"direction\":\"down\",\"row\":0,\"col\":2,\"length\":3},\
             {\"number\":3,\"direction\":\"across\",\"row\":2,\"col\":0,\"length\":3}\
             ]"
        );
    }
}
