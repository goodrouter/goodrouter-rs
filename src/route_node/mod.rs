pub mod route_node_merge;
pub mod route_node_rc;
pub mod route_node_utility;

use route_node_utility::*;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::BTreeSet,
    rc::{Rc, Weak},
};

pub type RouteNodeRc<'a> = Rc<RefCell<RouteNode<'a>>>;
type RouteNodeWeak<'a> = Weak<RefCell<RouteNode<'a>>>;

#[derive(Debug, Default)]
pub struct RouteNode<'a> {
    // the route's name, if any
    pub route_name: Option<&'a str>,
    // the route parameter names
    pub route_parameter_names: Vec<&'a str>,
    // suffix that comes after the parameter value (if any!) of the path
    anchor: &'a str,
    // does this node has a parameter
    has_parameter: bool,
    // children that represent the rest of the path that needs to be matched
    children: BTreeSet<RouteNodeRc<'a>>,
    // parent node, should only be null for the root node
    parent: Option<RouteNodeWeak<'a>>,
}

impl<'a> Ord for RouteNode<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.anchor.len() < other.anchor.len() {
            return Ordering::Greater;
        }
        if self.anchor.len() > other.anchor.len() {
            return Ordering::Less;
        }

        if !self.has_parameter && other.has_parameter {
            return Ordering::Less;
        }
        if self.has_parameter && !other.has_parameter {
            return Ordering::Greater;
        }

        if self.anchor < other.anchor {
            return Ordering::Less;
        }
        if self.anchor > other.anchor {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}

impl<'a> PartialOrd for RouteNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for RouteNode<'a> {}

impl<'a> PartialEq for RouteNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.anchor == other.anchor && self.has_parameter == other.has_parameter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::iter::FromIterator;

    #[test]
    fn route_ordering() {
        let nodes = vec![
            RouteNode {
                has_parameter: false,
                anchor: "aa",
                ..Default::default()
            },
            RouteNode {
                has_parameter: false,
                anchor: "xx",
                ..Default::default()
            },
            RouteNode {
                has_parameter: true,
                anchor: "aa",
                ..Default::default()
            },
            RouteNode {
                has_parameter: false,
                anchor: "x",
                ..Default::default()
            },
        ];

        let nodes_expected = nodes.iter();
        let nodes_actual = nodes.iter().sorted();

        assert_eq!(Vec::from_iter(nodes_actual), Vec::from_iter(nodes_expected));
    }
}
