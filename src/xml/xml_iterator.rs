use roxmltree::{Children, Node};

pub struct XmlIterator<'a> {
    descendants: Children<'a, 'a>,
    sub_iter: Option<Box<XmlIterator<'a>>>,
    offset: (f64, f64),
}

impl<'a> Iterator for XmlIterator<'a> {
    type Item = (Node<'a, 'a>, (f64, f64));

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sub_iter) = self.sub_iter.as_mut() {
            let next = sub_iter.next();
            if next.is_some() {
                return next;
            }
        }

        if let Some(next_node) = self.descendants.next() {
            let offset = next_node
                .attribute("transform")
                .and_then(|t| {
                    regex::Regex::new(
                        r"translate\(\s*(-?(?:\d+(?:\.\d+)?(?:e-?\d+)?))\s*(?:,\s*(-?(?:\d+(?:\.\d+)?(?:e-?\d+)?))\s*)?\)",
                    )
                    .unwrap()
                    .captures(t)
                })
                .map(|h| {
                    let mut iter = h.iter();
                    iter.next().unwrap().unwrap();
                    let first_num: f64 = iter.next().unwrap().unwrap().as_str().parse().unwrap();
                    let second_num: f64 = iter
                        .next()
                        .unwrap()
                        .map(|s| s.as_str())
                        .unwrap_or("0.0")
                        .parse()
                        .unwrap();
                    (first_num + self.offset.0, second_num + self.offset.1)
                })
                .unwrap_or(self.offset);
            self.sub_iter = Some(Box::new(XmlIterator {
                sub_iter: None,
                descendants: next_node.children(),
                offset,
            }));
            return Some((next_node, offset));
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
            offset: (0.0, 0.0),
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
