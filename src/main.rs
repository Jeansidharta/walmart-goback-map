use std::error::Error;

use clap::Parser;

mod vec2;
mod xml;

#[derive(Parser)]
struct Args {}

fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();
    let svg_file = include_str!("./plant.svg");
    let xml = roxmltree::Document::parse(svg_file)?;
    println!("Parsing route...");
    let route = xml::route::parse_route(&xml)?;
    std::fs::write("./route.json", serde_json::to_string(&route).unwrap()).unwrap();
    println!("Parsing shelf positions...");
    let all_positions = xml::parse_shelves(&xml, &route)?;
    std::fs::write(
        "./positions.json",
        serde_json::to_string(&all_positions).unwrap(),
    )
    .unwrap();

    Ok(())
}
