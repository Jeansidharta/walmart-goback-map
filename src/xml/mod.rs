use std::collections::HashMap;

use serde::Serialize;

mod xml_iterator;

trait NodeExt {
    fn iter(&self) -> xml_iterator::XmlIterator;
}

impl<'a> NodeExt for roxmltree::Node<'a, 'a> {
    fn iter(&self) -> xml_iterator::XmlIterator<'a> {
        xml_iterator::XmlIterator::new(self)
    }
}

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

pub fn parse_shelves(raw_xml: &str) -> anyhow::Result<HashMap<String, Corridor>> {
    let xml = roxmltree::Document::parse(raw_xml)?;

    let root = xml.root();
    let mut hash_map = root
        .iter()
        .filter(|n| {
            n.tag_name().name() == "circle"
                && n.attributes()
                    .any(|attr| attr.name() == "is-position" && attr.value() == "true")
        })
        .map(|node| -> anyhow::Result<NodePosition> {
            let position_name = node
                .attributes()
                .find(|attr| attr.name() == "label")
                .unwrap()
                .value();

            let mut split = position_name.split('-');
            let corridor_name = split.next().unwrap().to_string();
            let shelf_name = split.next().unwrap().to_string();
            let x = node.attribute("cx").unwrap().parse()?;
            let y = node.attribute("cy").unwrap().parse()?;
            Ok(NodePosition {
                corridor_name,
                shelf_name,
                x,
                y,
            })
        })
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
