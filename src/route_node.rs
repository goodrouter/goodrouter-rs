use crate::{path::parse_template_parts, route::Route, string::find_common_prefix_length};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    rc::{Rc, Weak},
};

pub type RouteNodeRc<'a> = Rc<RefCell<RouteNode<'a>>>;
pub type RouteNodeWeak<'a> = Weak<RefCell<RouteNode<'a>>>;

#[derive(Debug)]
enum InsertStrategy<'a> {
    Intermediate(&'a str),
    Merge,
    AddToSelf,
    AddToOther,
}

#[derive(Debug, Default)]
pub struct RouteNode<'a> {
    // name that identifies the route
    name: Option<&'a str>,
    // suffix that comes after the parameter value (if any!) of the path
    anchor: &'a str,
    // parameter name or null if this node does not represent a prameter
    parameter: Option<&'a str>,
    // children that represent the rest of the path that needs to be matched
    children: BTreeSet<RouteNodeRc<'a>>,
    // parent node, should only be null for the root node
    parent: Option<RouteNodeWeak<'a>>,
}

impl<'a> RouteNode<'a> {
    pub fn find_similar_child(&self, other: &RouteNode<'a>) -> Option<(RouteNodeRc<'a>, usize)> {
        let chars_other: Vec<_> = other.anchor.chars().collect();

        for child in self.children.iter() {
            let child_borrow = child.borrow();

            if child_borrow.parameter.is_some() {
                continue;
            }
            if child_borrow.name.is_some() {
                continue;
            }

            let chars_child: Vec<_> = child_borrow.anchor.chars().collect();

            let common_prefix_length = find_common_prefix_length(&chars_other, &chars_child);

            if common_prefix_length == 0 {
                continue;
            }

            return Some((child.clone(), common_prefix_length));
        }

        return None;
    }

    fn get_insert_strategy(
        &self,
        node_other: &Self,
        common_prefix_length: usize,
    ) -> InsertStrategy<'a> {
        let common_prefix = &self.anchor[..common_prefix_length];

        if self.anchor == node_other.anchor {
            if self.name.is_some() && node_other.name.is_some() && self.name != node_other.name {
                panic!("ambiguous route");
            } else if self.parameter.is_some()
                && node_other.parameter.is_some()
                && self.parameter != node_other.parameter
            {
                // this is kind of an edge case, the parameter names are different, but for the rest
                // the node is the same. We place an intermediate node that groups the two with an
                // empty anchor.

                return InsertStrategy::Intermediate(common_prefix);
            } else {
                // The two nodes can be merged! This is great! we merge the names, parameter
                // and the children

                return InsertStrategy::Merge;
            }
        } else if self.anchor == common_prefix {
            // in this case the similar node should be child of the new node
            // because the new node anchor is a prefix of the similar node anchor

            return InsertStrategy::AddToSelf;
        } else if node_other.anchor == common_prefix {
            // this is the exact inverse of the previous clause

            return InsertStrategy::AddToOther;
        } else {
            // we encountered two nodes that are not the same, and none of the two nodes
            // has an anchor that is a prefix of the other. Both nodes share a common prefix
            // in the anchor. We need an intermediate that has the common prefix as the
            // anchor with the two nodes as children. The common prefix is removed from the
            // anchor of the two nodes.

            return InsertStrategy::Intermediate(common_prefix);
        }
    }

    pub fn new_chain(name: &'a str, template: &'a str) -> RouteNodeNewChain<'a> {
        return RouteNodeNewChain::new(name, template);
    }
}
impl<'a> Iterator for RouteNodeNewChain<'a> {
    type Item = RouteNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.parts.is_empty() {
            return None;
        }

        let anchor = self.parts.pop().unwrap();
        let parameter = self.parts.pop();

        let node_new = RouteNode {
            anchor,
            name: self.name,
            parameter,
            children: BTreeSet::default(),
            parent: None,
        };

        self.name = None;

        return Some(node_new);
    }
}

impl<'a> Ord for RouteNode<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.parameter.is_none() < other.parameter.is_none() {
            return Ordering::Greater;
        }
        if self.parameter.is_none() > other.parameter.is_none() {
            return Ordering::Less;
        }

        if self.name.is_none() < other.name.is_none() {
            return Ordering::Greater;
        }
        if self.name.is_none() > other.name.is_none() {
            return Ordering::Less;
        }

        if self.anchor.len() < other.anchor.len() {
            return Ordering::Greater;
        }
        if self.anchor.len() > other.anchor.len() {
            return Ordering::Less;
        }

        if self.anchor > other.anchor {
            return Ordering::Greater;
        }
        if self.anchor < other.anchor {
            return Ordering::Less;
        }

        return Ordering::Equal;
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
        self.name == other.name && self.anchor == other.anchor && self.parameter == other.parameter
    }
}

pub trait RouteNodeRcExt<'a> {
    fn insert(&self, name: &'a str, template: &'a str) -> RouteNodeRc<'a>;

    fn stringify(&self, parameters: &HashMap<String, String>) -> String;
    fn parse(&self, path: &str, parameters: HashMap<String, String>) -> Option<Route>;

    fn chain(&self) -> RouteNodeRcChain<'a>;
}

impl<'a> RouteNodeRcExt<'a> for RouteNodeRc<'a> {
    fn insert(&self, name: &'a str, template: &'a str) -> Self {
        let mut chain: Vec<_> = RouteNode::new_chain(name, template).collect();
        chain.reverse();

        let mut node_current_rc = self.clone();
        for mut node_chain in chain {
            let similar_child = node_current_rc.borrow().find_similar_child(&node_chain);
            if let Some((node_similar_rc, prefix_length)) = similar_child {
                let strategy = node_similar_rc
                    .borrow()
                    .get_insert_strategy(&node_chain, prefix_length);

                match strategy {
                    InsertStrategy::Merge => {
                        let mut node_similar = node_similar_rc.borrow_mut();

                        if node_similar.name.is_none() {
                            node_similar.name = node_chain.name;
                        }

                        if node_similar.parameter.is_none() {
                            node_similar.parameter = node_chain.parameter;
                        }

                        node_similar
                            .children
                            .append(&mut node_chain.children.clone());

                        node_current_rc = node_similar_rc.clone();
                    }
                    InsertStrategy::AddToSelf => {
                        node_chain.anchor = &node_chain.anchor[prefix_length..];
                        node_chain.parent = Some(Rc::downgrade(&node_similar_rc));

                        let node_chain_rc = Rc::new(RefCell::new(node_chain));
                        {
                            let mut node_similar = node_similar_rc.borrow_mut();
                            if let Some(node_child_rc) = node_similar.children.get(&node_chain_rc) {
                                node_current_rc = node_child_rc.clone();
                            } else {
                                assert!(node_similar.children.insert(node_chain_rc.clone()));
                                node_current_rc = node_chain_rc;
                            }
                        }
                    }
                    InsertStrategy::AddToOther => {
                        let node_chain_rc = Rc::new(RefCell::new(node_chain));
                        {
                            let mut node_current = node_current_rc.borrow_mut();

                            assert!(node_current.children.remove(&node_similar_rc));
                            assert!(node_current.children.insert(node_chain_rc.clone()));
                        }

                        {
                            let mut node_similar = node_similar_rc.borrow_mut();

                            node_similar.anchor = &node_similar.anchor[prefix_length..];
                            node_similar.parent = Some(Rc::downgrade(&node_chain_rc));
                        }

                        {
                            let mut node_chain = node_chain_rc.borrow_mut();

                            assert!(node_chain.children.insert(node_similar_rc.clone()));
                        }

                        node_current_rc = node_similar_rc;
                    }
                    InsertStrategy::Intermediate(common_prefix) => {
                        let node_chain_rc = Rc::new(RefCell::new(node_chain));

                        let mut node_intermediate = RouteNode {
                            anchor: common_prefix,
                            name: None,
                            parameter: None,
                            children: BTreeSet::default(),
                            parent: Some(Rc::downgrade(&node_current_rc)),
                        };
                        assert!(node_intermediate.children.insert(node_similar_rc.clone()));
                        assert!(node_intermediate.children.insert(node_chain_rc.clone()));

                        let node_intermediate_rc = Rc::new(RefCell::new(node_intermediate));
                        {
                            let mut node_current = node_current_rc.borrow_mut();
                            assert!(node_current.children.remove(&node_similar_rc));
                            assert!(node_current.children.insert(node_intermediate_rc.clone()));
                        }

                        {
                            let mut node_chain = node_chain_rc.borrow_mut();
                            let mut node_similar = node_similar_rc.borrow_mut();

                            node_chain.parent = Some(Rc::downgrade(&node_intermediate_rc));
                            node_similar.parent = Some(Rc::downgrade(&node_intermediate_rc));

                            node_chain.anchor = &node_chain.anchor[prefix_length..];
                            node_similar.anchor = &node_similar.anchor[prefix_length..];
                        }

                        node_current_rc = node_chain_rc;
                    }
                };
            } else {
                let mut node_child = node_chain;
                node_child.parent = Some(Rc::downgrade(&node_current_rc));

                let node_child_rc = Rc::new(RefCell::new(node_child));
                {
                    let mut node_current = node_current_rc.borrow_mut();
                    assert!(node_current.children.insert(node_child_rc.clone()));
                }
                node_current_rc = node_child_rc;
            }
        }

        return node_current_rc;
    }

    fn chain(&self) -> RouteNodeRcChain<'a> {
        return RouteNodeRcChain {
            node: Some(self.clone()),
        };
    }

    fn stringify(&self, parameters: &HashMap<String, String>) -> String {
        let mut path = String::new();
        let mut chain: Vec<_> = self.clone().chain().collect();
        chain.reverse();

        for node in chain {
            let node_borrow = node.borrow();
            if let Some(parameter) = node_borrow.parameter {
                if let Some(value) = parameters.get(parameter) {
                    path += value;
                };
            }
            path += node_borrow.anchor;
        }

        return path;
    }

    fn parse(&self, path: &str, parameters: HashMap<String, String>) -> Option<Route> {
        let mut path = path;
        let mut parameters = parameters;

        let self_borrow = self.borrow();
        if let Some(parameter) = self_borrow.parameter {
            // we are matching a parameter value! If the path's length is 0, there is no match, because a parameter value should have at least length 1
            if path.is_empty() {
                return None;
            }

            // look for the anchor in the path. If the anchor is empty, match the remainder of the path
            let index = if self_borrow.anchor.is_empty() {
                Some(path.len())
            } else {
                path.find(self_borrow.anchor)
            };

            if let Some(index) = index {
                let value = &path[..index];

                // remove the matches part from the path
                path = &path[index + self_borrow.anchor.len()..];

                assert_eq!(
                    parameters.insert(parameter.to_owned(), value.to_owned()),
                    None
                );
            } else {
                return None;
            }
        } else {
            // if this node does not represent a parameter we expect the path to start with the `anchor`
            if !path.starts_with(self_borrow.anchor) {
                // this node does not match the path
                return None;
            }

            // we successfully matches the node to the path, now remove the matched part from the path
            path = &path[self_borrow.anchor.len()..];
        }

        for child in &self_borrow.children {
            if let Some(route) = child.parse(path, parameters.clone()) {
                return Some(route);
            }
        }

        // if the node had a route name and there is no path left to match against then we found a route
        if path.is_empty() {
            if let Some(name) = self_borrow.name {
                return Some(Route {
                    name: name.to_owned(),
                    parameters,
                });
            }
        }

        return None;
    }
}

pub struct RouteNodeNewChain<'a> {
    name: Option<&'a str>,
    parts: Vec<&'a str>,
}

impl<'a> RouteNodeNewChain<'a> {
    pub fn new(name: &'a str, template: &'a str) -> Self {
        let parts: Vec<_> = parse_template_parts(template).collect();
        Self {
            name: Some(name),
            parts,
        }
    }
}

pub struct RouteNodeRcChain<'a> {
    node: Option<RouteNodeRc<'a>>,
}

impl<'a> Iterator for RouteNodeRcChain<'a> {
    type Item = RouteNodeRc<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.clone();

        if let Some(node) = self.node.clone() {
            let parent_node = &node.borrow().parent;
            if let Some(parent_node) = parent_node {
                self.node = Some(parent_node.upgrade().unwrap());
            } else {
                self.node = None;
            }
        }

        return node;
    }
}

mod tests {
    use super::*;

    #[test]
    fn route_node_flow() {
        let node_root = RouteNode::default();
        let node_root_rc = Rc::new(RefCell::new(node_root));

        node_root_rc.insert("a", "/a");
        node_root_rc.insert("b", "/b/{x}");
        node_root_rc.insert("c", "/b/{x}/c");
        node_root_rc.insert("d", "/b/{x}/d");

        {
            let node_root = node_root_rc.borrow();
            assert_eq!(node_root.children.len(), 2);
        }

        let route = node_root_rc.parse("/a", HashMap::default()).unwrap();
        assert_eq!(route.name, "a");

        let route = node_root_rc.parse("/b/x", HashMap::default()).unwrap();
        assert_eq!(route.name, "b");

        let route = node_root_rc.parse("/b/y/c", HashMap::default()).unwrap();
        assert_eq!(route.name, "c");

        let route = node_root_rc.parse("/b/z/d", HashMap::default()).unwrap();
        assert_eq!(route.name, "d");
    }
}
