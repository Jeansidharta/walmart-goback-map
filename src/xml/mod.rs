use std::collections::HashMap;

use roxmltree::Document;
use serde::Serialize;

mod xml_iterator;
use xml_iterator::NodeExt;

#[derive(Debug, Serialize)]
pub struct NodePosition {
    corridor_name: String,
    shelf_name: String,
    x: f64,
    y: f64,
}

#[derive(Default, Debug, Serialize)]
pub struct Corridor {
    pub average_position: (f64, f64),
    pub name: String,
    pub section: String,
    pub shelves: Vec<NodePosition>,
}

pub fn parse_shelves(xml: &Document) -> anyhow::Result<HashMap<String, Corridor>> {
    let mut hash_map = xml
        .root()
        .iter()
        .filter(|(n, _)| {
            n.tag_name().name() == "circle"
                && n.attributes()
                    .any(|attr| attr.name() == "is-position" && attr.value() == "true")
        })
        .map(
            |(node, (offset_x, offset_y))| -> anyhow::Result<NodePosition> {
                let position_name = node
                    .attributes()
                    .find(|attr| attr.name() == "label")
                    .unwrap()
                    .value();

                let mut split = position_name.split('-');
                let corridor_name = split.next().unwrap().to_string();
                let shelf_name = split.next().unwrap().to_string();
                let x = node.attribute("cx").unwrap().parse::<f64>()? + offset_x;
                let y = node.attribute("cy").unwrap().parse::<f64>()? + offset_y;
                Ok(NodePosition {
                    corridor_name,
                    shelf_name,
                    x,
                    y,
                })
            },
        )
        .collect::<anyhow::Result<Vec<NodePosition>>>()?
        .into_iter()
        .fold(HashMap::<String, Corridor>::new(), |mut map, position| {
            if let Some(corridor) = map.get_mut(&position.corridor_name) {
                corridor.shelves.push(position);
            } else {
                map.insert(
                    position.corridor_name.clone(),
                    Corridor {
                        name: position.corridor_name.clone(),
                        shelves: vec![position],
                        ..Default::default()
                    },
                );
            }
            map
        });
    hash_map.values_mut().for_each(|corridor| {
        let avg_x =
            corridor.shelves.iter().fold(0f64, |sum, s| sum + s.x) / corridor.shelves.len() as f64;
        let avg_y =
            corridor.shelves.iter().fold(0f64, |sum, s| sum + s.y) / corridor.shelves.len() as f64;
        corridor.average_position = (avg_x, avg_y);
    });
    Ok(hash_map)
}
