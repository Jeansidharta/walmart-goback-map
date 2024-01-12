use std::error::Error;

use clap::Parser;

mod xml;

#[derive(Parser)]
struct Args {}

fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();
    let svg_file = include_str!("./plant-ungroup.svg");
    let all_positions = xml::parse_shelves(svg_file)?;
    std::fs::write(
        "./positions.json",
        serde_json::to_string(&all_positions).unwrap(),
    )
    .unwrap();

    Ok(())
}
