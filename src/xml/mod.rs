use std::collections::HashMap;

use roxmltree::Document;
use serde::Serialize;

pub mod route;
mod xml_iterator;
use xml_iterator::NodeExt;

use crate::vec2::Vec2;

use self::route::{Connection, Route};

#[derive(Debug, Serialize, Default)]
pub struct RouteProjection {
    connection: Connection,
    t: f64,
    point: Vec2,
}

impl RouteProjection {
    pub fn new(connection: Connection, point: Vec2, route: &Route) -> Self {
        let p1 = &route.points[connection.i1];
        let p2 = &route.points[connection.i2];
        let (point, t) = point.project_line_segment((p1, p2));
        RouteProjection {
            connection,
            point,
            t,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShelfPosition {
    corridor: String,
    shelf: String,
    x: f64,
    y: f64,
    route_projection: RouteProjection,
}

#[derive(Default, Debug, Serialize)]
pub struct Corridor {
    pub average_position: Vec2,
    pub name: String,
    pub section: String,
    pub shelves: HashMap<String, ShelfPosition>,
    pub route_projection: RouteProjection,
}

const INKSCAPE_SCOPE: &str = "http://www.inkscape.org/namespaces/inkscape";

fn distance_line_segment_point_sqr(line_segment: (&Vec2, &Vec2), point: &Vec2) -> f64 {
    let (proj, _) = point.project_line_segment(line_segment);
    point.subtract(&proj).modulus_sqr()
}

pub fn parse_shelves(xml: &Document, route: &Route) -> anyhow::Result<HashMap<String, Corridor>> {
    let mut hash_map = xml
        .root()
        .iter()
        .filter(|(n, _)| {
            n.attributes()
                .any(|attr| attr.name() == "is-position" && attr.value() == "true")
        })
        .map(|(node, transforms)| -> anyhow::Result<ShelfPosition> {
            let (corridor, shelf) = {
                let node_label = node
                    .attributes()
                    .find(|attr| attr.name() == "label")
                    .unwrap()
                    .value();

                let mut split = node_label.split('-');
                let corridor = split.next().unwrap().to_string();
                let shelf = split.next().unwrap().to_string();
                (corridor, shelf)
            };

            let point = match node.tag_name().name() {
                "rect" => {
                    let width = node.attribute("width").unwrap().parse::<f64>()?;
                    let height = node.attribute("height").unwrap().parse::<f64>()?;

                    transforms.apply(Vec2 {
                        x: node.attribute("x").unwrap().parse::<f64>()? + width / 2.0,
                        y: node.attribute("y").unwrap().parse::<f64>()? + height / 2.0,
                    })
                }
                "circle" => transforms.apply(Vec2 {
                    x: node.attribute("cx").unwrap().parse::<f64>()?,
                    y: node.attribute("cy").unwrap().parse::<f64>()?,
                }),
                _ => unimplemented!(),
            };

            let (connection, _) = route.connections.iter().fold(
                (route.connections[0], std::f64::MAX),
                |acc,
                 Connection {
                     i1: p1_index,
                     i2: p2_index,
                 }| {
                    let p1 = &route.points[*p1_index];
                    let p2 = &route.points[*p2_index];
                    let new_dist = distance_line_segment_point_sqr((p1, p2), &point);
                    if new_dist < acc.1 {
                        (
                            Connection {
                                i1: *p1_index,
                                i2: *p2_index,
                            },
                            new_dist,
                        )
                    } else {
                        acc
                    }
                },
            );

            let route_projection = RouteProjection::new(connection, point.clone(), route);

            Ok(ShelfPosition {
                route_projection,
                corridor,
                shelf,
                x: point.x,
                y: point.y,
            })
        })
        .collect::<anyhow::Result<Vec<ShelfPosition>>>()?
        .into_iter()
        .fold(HashMap::<String, Corridor>::new(), |mut map, position| {
            if let Some(corridor) = map.get_mut(&position.corridor) {
                corridor.shelves.insert(position.shelf.clone(), position);
            } else {
                let mut shelves = HashMap::new();
                let corridor_name = position.corridor.clone();
                shelves.insert(position.shelf.clone(), position);
                map.insert(
                    corridor_name.clone(),
                    Corridor {
                        name: corridor_name,
                        shelves,
                        ..Default::default()
                    },
                );
            }
            map
        });
    hash_map.values_mut().for_each(|corridor| {
        let avg_x = corridor.shelves.iter().fold(0f64, |sum, s| sum + s.1.x)
            / corridor.shelves.len() as f64;
        let avg_y = corridor.shelves.iter().fold(0f64, |sum, s| sum + s.1.y)
            / corridor.shelves.len() as f64;
        corridor.average_position = Vec2 { x: avg_x, y: avg_y };

        let (connection, _) = route.connections.iter().fold(
            (route.connections[0], std::f64::MAX),
            |acc,
             Connection {
                 i1: p1_index,
                 i2: p2_index,
             }| {
                let p1 = &route.points[*p1_index];
                let p2 = &route.points[*p2_index];
                let new_dist =
                    distance_line_segment_point_sqr((p1, p2), &corridor.average_position);
                if new_dist < acc.1 {
                    (
                        Connection {
                            i1: *p1_index,
                            i2: *p2_index,
                        },
                        new_dist,
                    )
                } else {
                    acc
                }
            },
        );

        corridor.route_projection =
            RouteProjection::new(connection, corridor.average_position.clone(), route);
    });
    Ok(hash_map)
}
