use roxmltree::{Children, Node};

pub struct XmlIterator<'a> {
    descendants: Children<'a, 'a>,
    sub_iter: Option<Box<XmlIterator<'a>>>,
}

impl<'a> Iterator for XmlIterator<'a> {
    type Item = Node<'a, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sub_iter) = self.sub_iter.as_mut() {
            let next = sub_iter.next();
            if next.is_some() {
                return next;
            }
        }

        if let Some(next_node) = self.descendants.next() {
            self.sub_iter = Some(Box::new(XmlIterator {
                sub_iter: None,
                descendants: next_node.children(),
            }));
            return Some(next_node);
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
        }
    }
}
