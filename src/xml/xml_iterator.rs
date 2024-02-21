use std::f64::consts::PI;

use roxmltree::{Children, Node};
use svgtypes::{TransformListParser, TransformListToken};

use crate::vec2::Vec2;

pub struct XmlIterator<'a> {
    descendants: Children<'a, 'a>,
    sub_iter: Option<Box<XmlIterator<'a>>>,
    transforms: TransformList,
}

#[derive(Debug, Clone, Default)]
pub struct TransformList {
    transforms: Vec<TransformListToken>,
}

impl TransformList {
    pub fn from(transforms: Vec<TransformListToken>) -> Self {
        TransformList { transforms }
    }

    pub fn apply(&self, vec: Vec2) -> Vec2 {
        self.transforms
            .iter()
            .rev()
            .fold(vec, |Vec2 { x, y }, transform| match transform {
                TransformListToken::Translate { tx, ty } => Vec2 {
                    x: x + tx,
                    y: y + ty,
                },
                TransformListToken::Matrix { a, b, c, d, e, f } => Vec2 {
                    x: x * a + y * c + e,
                    y: x * b + y * d + f,
                },
                TransformListToken::Scale { sx, sy } => Vec2 {
                    x: x * sx,
                    y: y * sy,
                },
                TransformListToken::Rotate { angle } => {
                    let angle = angle * PI / 180.0;
                    let cos = angle.cos();
                    let sin = angle.sin();
                    Vec2 {
                        x: x * cos - y * sin,
                        y: x * sin + y * cos,
                    }
                }
                TransformListToken::SkewX { angle } => Vec2 {
                    x: x + y * (angle * PI / 180.0).tan(),
                    y,
                },
                TransformListToken::SkewY { angle } => Vec2 {
                    x,
                    y: x * (angle * PI / 180.0).tan() + y,
                },
            })
    }

    pub fn append(&mut self, list: &mut TransformList) -> &mut Self {
        self.transforms.append(&mut list.transforms);
        self
    }
}

impl<'a> Iterator for XmlIterator<'a> {
    type Item = (Node<'a, 'a>, TransformList);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sub_iter) = self.sub_iter.as_mut() {
            let next = sub_iter.next();
            if next.is_some() {
                return next;
            }
        }

        if let Some(next_node) = self.descendants.next() {
            let mut new_transforms = next_node
                .attribute("transform")
                .map(|t| {
                    TransformList::from(
                        TransformListParser::from(t)
                            .filter_map(|t| t.ok())
                            .collect::<Vec<TransformListToken>>(),
                    )
                })
                .unwrap_or_default();
            let mut transforms = self.transforms.clone();
            transforms.append(&mut new_transforms);
            self.sub_iter = Some(Box::new(XmlIterator {
                sub_iter: None,
                descendants: next_node.children(),
                transforms: transforms.clone(),
            }));

            return Some((next_node, transforms));
        }

        None
    }
}

impl<'a> XmlIterator<'a> {
    pub fn new(node: &roxmltree::Node<'a, 'a>) -> Self {
        let descendants = node.children();
        XmlIterator {
            descendants,
            sub_iter: None,
            transforms: TransformList::default(),
        }
    }
}

pub trait NodeExt {
    fn iter(&self) -> XmlIterator;
}

impl<'a> NodeExt for roxmltree::Node<'a, 'a> {
    fn iter(&self) -> XmlIterator<'a> {
        XmlIterator::new(self)
    }
}
