use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use gridslots::{parse_grid, slots, Direction};

fn main() -> ExitCode {
    // No file argument, or "-", means read the grid from stdin so the
    // tool can sit in a pipeline instead of always needing a temp file.
    let text = match env::args().nth(1) {
        Some(path) if path != "-" => match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("could not read {path}: {err}");
                return ExitCode::FAILURE;
            }
        },
        _ => {
            let mut text = String::new();
            match io::stdin().read_to_string(&mut text) {
                Ok(_) => text,
                Err(err) => {
                    eprintln!("could not read stdin: {err}");
                    return ExitCode::FAILURE;
                }
            }
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
