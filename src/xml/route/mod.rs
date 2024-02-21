use crate::xml::xml_iterator::NodeExt;
use crate::xml::INKSCAPE_SCOPE;
use std::collections::HashMap;

use roxmltree::Document;
use serde::Serialize;

use crate::vec2::Vec2;

#[derive(Debug, Serialize)]
pub struct Route {
    start: usize,
    pub points: Vec<Vec2>,
    pub connections_dict: HashMap<usize, Vec<usize>>,
    pub connections: Vec<Connection>,
}

#[derive(Default, Debug, Serialize, Clone, Copy)]
pub struct Connection {
    pub i1: usize,
    pub i2: usize,
}

pub fn parse_route(xml: &Document) -> anyhow::Result<Route> {
    let root = xml.root();
    let (route_group, transform) = root
        .iter()
        .find(|(n, _)| {
            n.tag_name().name() == "g"
                && n.attribute((INKSCAPE_SCOPE, "label"))
                    .is_some_and(|a| a == "Route")
        })
        .expect("No route group found");

    let mut start_index = 0;
    let points = route_group
        .children()
        .filter(|n| n.tag_name().name() == "circle")
        .enumerate()
        .map(|(index, n)| {
            let x = n.attribute("cx").unwrap().parse::<f64>().unwrap();
            let y = n.attribute("cy").unwrap().parse::<f64>().unwrap();
            if n.attribute((INKSCAPE_SCOPE, "label"))
                .is_some_and(|v| v == "start")
            {
                start_index = index;
            }
            transform.apply(Vec2 { x, y })
        })
        .collect::<Vec<Vec2>>();

    let mut connections_dict: HashMap<usize, Vec<usize>> = HashMap::new();

    let connections: Vec<Connection> = route_group
        .children()
        .filter(|n| n.tag_name().name() == "path")
        .map(|n| {
            let d = n.attribute("d").unwrap();
            let (_, path) = svg_path_parser::parse(d).next().unwrap();
            if path.len() > 2 {
                panic!("Path has more than two points");
            }
            let start = transform.apply(Vec2 {
                x: path[0].0,
                y: path[0].1,
            });
            let end = transform.apply(Vec2 {
                x: path[1].0,
                y: path[1].1,
            });

            let (start_point, _) = points
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

            let (end_point, _) = points
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
            (start_point, end_point)
        })
        .map(|(i1, i2)| Connection { i1, i2 })
        .collect();
    connections
        .iter()
        .for_each(|Connection { i1: start, i2: end }| {
            if let Some(container) = connections_dict.get_mut(start) {
                container.push(*end);
            } else {
                connections_dict.insert(*start, vec![*end]);
            }

            if let Some(container) = connections_dict.get_mut(end) {
                container.push(*start);
            } else {
                connections_dict.insert(*end, vec![*start]);
            }
        });

    Ok(Route {
        points,
        start: start_index,
        connections,
        connections_dict,
    })
}
