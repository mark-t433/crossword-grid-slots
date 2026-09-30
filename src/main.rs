use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use gridslots::{asymmetric_blocks, parse_grid, slots, to_json, Direction};

enum Format {
    Text,
    Json,
}

fn parse_format(value: &str) -> Option<Format> {
    match value {
        "text" => Some(Format::Text),
        "json" => Some(Format::Json),
        _ => None,
    }
}

fn main() -> ExitCode {
    let mut format = Format::Text;
    let mut path = None;
    let mut warn_symmetry = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--warn-symmetry" {
            warn_symmetry = true;
            continue;
        }

        let value = if let Some(value) = arg.strip_prefix("--format=") {
            Some(value.to_string())
        } else if arg == "--format" {
            match args.next() {
                Some(value) => Some(value),
                None => {
                    eprintln!("--format needs a value");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            None
        };

        match value {
            Some(value) => match parse_format(&value) {
                Some(f) => format = f,
                None => {
                    eprintln!("unknown format {value:?}, expected \"text\" or \"json\"");
                    return ExitCode::FAILURE;
                }
            },
            None => path = Some(arg),
        }
    }

    // No file argument, or "-", means read the grid from stdin so the
    // tool can sit in a pipeline instead of always needing a temp file.
    let text = match path {
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

    // Warnings go to stderr so json or text output stays clean to pipe,
    // and the exit code stays success: an asymmetric grid is still valid.
    if warn_symmetry {
        for (row, col) in asymmetric_blocks(&grid) {
            eprintln!(
                "warning: block at row={row} col={col} has no partner at row={} col={}",
                grid.rows() - 1 - row,
                grid.cols() - 1 - col,
            );
        }
    }

    let result = slots(&grid);

    match format {
        Format::Json => println!("{}", to_json(&result)),
        Format::Text => {
            for slot in result {
                let dir = match slot.direction {
                    Direction::Across => "A",
                    Direction::Down => "D",
                };
                println!(
                    "{number}{dir} row={row} col={col} len={len} text={text}",
                    number = slot.number,
                    row = slot.row,
                    col = slot.col,
                    len = slot.length,
                    text = slot.text,
                );
            }
        }
    }

    ExitCode::SUCCESS
}
