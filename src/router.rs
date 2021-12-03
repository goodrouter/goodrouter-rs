use std::collections::HashMap;

use crate::{
    route::Route,
    route_node::{RouteNodeRc, RouteNodeRcExt},
};

pub struct Router<'a> {
    node_root: RouteNodeRc<'a>,
    node_leafs: HashMap<&'a str, RouteNodeRc<'a>>,
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Self {
            node_root: RouteNodeRc::default(),
            node_leafs: HashMap::new(),
        }
    }

    pub fn insert_route(&mut self, name: &'a str, template: &'a str) {
        let node_leaf = self.node_root.insert(name, template);
        self.node_leafs.insert(name, node_leaf);
    }

    pub fn parse_route(&self, path: &'a str) -> Option<Route> {
        let parameters = HashMap::new();
        return self.node_root.parse(path, parameters);
    }

    pub fn stringify_route(&self, route: &'a Route) -> Option<String> {
        if let Some(node) = self.node_leafs.get(route.name.as_str()) {
            let node = node.clone();

            return Some(node.stringify(&route.parameters));
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router() {
        let mut router = Router::new();

        router.insert_route("one", "/a");
        router.insert_route("two", "/a/{x}/{y}");
        router.insert_route("three", "/c/{x}");
        router.insert_route("four", "/c/{x}/{y}/");

        let route = router.parse_route("/a").unwrap();
        assert_eq!(route.name, "one");

        let route = router.parse_route("/a/1/2").unwrap();
        assert_eq!(route.name, "two");
        assert_eq!(
            route.parameters,
            vec![("x", "1"), ("y", "2"),]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        );

        let route = Route {
            name: "two".to_owned(),
            parameters: vec![("x", "1"), ("y", "2")]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        };
        let path = router.stringify_route(&route).unwrap();
        assert_eq!(path, "/a/1/2");

        let route = router.parse_route("/c/3").unwrap();
        assert_eq!(route.name, "three");
        assert_eq!(
            route.parameters,
            vec![("x", "3"),]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        );

        let route = router.parse_route("/c/3/4").unwrap();
        assert_eq!(route.name, "three");
        assert_eq!(
            route.parameters,
            vec![("x", "3/4"),]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        );

        let route = Route {
            name: "three".to_owned(),
            parameters: vec![("x", "3/4")]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        };
        let path = router.stringify_route(&route).unwrap();
        assert_eq!(path, "/c/3/4");

        let route = router.parse_route("/c/3/4/").unwrap();
        assert_eq!(route.name, "four");
        assert_eq!(
            route.parameters,
            vec![("x", "3"), ("y", "4"),]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        );
    }
}
