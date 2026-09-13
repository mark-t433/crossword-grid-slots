use std::env;
use std::fs;
use std::process::ExitCode;

use gridslots::{parse_grid, slots, Direction};

fn main() -> ExitCode {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: gridslots <grid-file>");
            return ExitCode::FAILURE;
        }
    };

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("could not read {path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let grid = match parse_grid(&text) {
        Ok(grid) => grid,
        Err(err) => {
            eprintln!("could not parse grid: {err}");
            return ExitCode::FAILURE;
        }
    };

    for slot in slots(&grid) {
        let dir = match slot.direction {
            Direction::Across => "A",
            Direction::Down => "D",
        };
        println!(
            "{number}{dir} row={row} col={col} len={len}",
            number = slot.number,
            row = slot.row,
            col = slot.col,
            len = slot.length,
        );
    }

    ExitCode::SUCCESS
}
