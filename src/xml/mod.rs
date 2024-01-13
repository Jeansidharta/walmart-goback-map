use std::collections::HashMap;

use roxmltree::Document;
use serde::Serialize;

mod xml_iterator;
use xml_iterator::NodeExt;

#[derive(Debug, Serialize)]
pub struct RouteProjection {
    point: (usize, usize),
    t: f64,
}

#[derive(Debug, Serialize)]
pub struct ShelfPosition {
    corridor_name: String,
    shelf_name: String,
    x: f64,
    y: f64,
    route_projection: RouteProjection,
}

#[derive(Debug, Serialize)]
pub struct NodePosition {
    x: f64,
    y: f64,
}

impl NodePosition {
    pub fn distance_sqr(&self, other: &Self) -> f64 {
        (other.x - self.x).powi(2) + (other.y - self.y).powi(2)
    }
}

#[derive(Default, Debug, Serialize)]
pub struct Corridor {
    pub average_position: (f64, f64),
    pub name: String,
    pub section: String,
    pub shelves: Vec<ShelfPosition>,
}

const INKSCAPE_SCOPE: &str = "http://www.inkscape.org/namespaces/inkscape";

#[derive(Debug, Serialize)]
pub struct Route {
    points: Vec<NodePosition>,
    connections_dict: HashMap<usize, Vec<usize>>,
    connections: Vec<(usize, usize)>,
}

pub fn parse_route(xml: &Document) -> anyhow::Result<Route> {
    let root = xml.root();
    let (route_group, (offset_x, offset_y)) = root
        .iter()
        .find(|(n, _)| {
            n.tag_name().name() == "g"
                && n.attribute((INKSCAPE_SCOPE, "label"))
                    .is_some_and(|a| a == "Route")
        })
        .expect("No route group found");

    let points = route_group
        .children()
        .filter(|n| n.tag_name().name() == "circle")
        .map(|n| {
            let x = n.attribute("cx").unwrap().parse::<f64>().unwrap() + offset_x;
            let y = n.attribute("cy").unwrap().parse::<f64>().unwrap() + offset_y;

            NodePosition { x, y }
        })
        .collect::<Vec<NodePosition>>();

    let mut connections_dict: HashMap<usize, Vec<usize>> = HashMap::new();

    let connections: Vec<(usize, usize)> = route_group
        .children()
        .filter(|n| n.tag_name().name() == "path")
        .map(|n| {
            let d = n.attribute("d").unwrap();
            let (_, path) = svg_path_parser::parse(d).next().unwrap();
            let start = NodePosition {
                x: path[0].0 + offset_x,
                y: path[0].1 + offset_y,
            };
            let end = NodePosition {
                x: path[1].0 + offset_x,
                y: path[1].1 + offset_y,
            };

            let start_point = points
                .iter()
                .enumerate()
                .reduce(|(index_left, pos_left), (index_right, pos_right)| {
                    if pos_left.distance_sqr(&start) < pos_right.distance_sqr(&start) {
                        (index_left, pos_left)
                    } else {
                        (index_right, pos_right)
                    }
                })
                .unwrap();

            let end_point = points
                .iter()
                .enumerate()
                .reduce(|(index_left, pos_left), (index_right, pos_right)| {
                    if pos_left.distance_sqr(&end) < pos_right.distance_sqr(&end) {
                        (index_left, pos_left)
                    } else {
                        (index_right, pos_right)
                    }
                })
                .unwrap();
            (start_point.0, end_point.0)
        })
        .collect();
    connections.iter().for_each(|(start, end)| {
        if let Some(container) = connections_dict.get_mut(&start) {
            container.push(*end);
        } else {
            connections_dict.insert(*start, vec![*end]);
        }

        if let Some(container) = connections_dict.get_mut(&end) {
            container.push(*end);
        } else {
            connections_dict.insert(*end, vec![*start]);
        }
    });

    Ok(Route {
        points,
        connections,
        connections_dict,
    })
}

pub fn parse_shelves(xml: &Document, route: &Route) -> anyhow::Result<HashMap<String, Corridor>> {
    let mut hash_map = xml
        .root()
        .iter()
        .filter(|(n, _)| {
            n.tag_name().name() == "circle"
                && n.attributes()
                    .any(|attr| attr.name() == "is-position" && attr.value() == "true")
        })
        .map(
            |(node, (offset_x, offset_y))| -> anyhow::Result<ShelfPosition> {
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
                Ok(ShelfPosition {
                    corridor_name,
                    shelf_name,
                    x,
                    y,
                })
            },
        )
        .collect::<anyhow::Result<Vec<ShelfPosition>>>()?
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
