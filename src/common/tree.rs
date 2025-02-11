use crate::common::node::Node;

pub struct Tree<T> {
    root: Node<T>,
    children: Vec<Node<T>>,
}
