use crate::{
    route_node::route_node_rc::{route_node_insert, route_node_parse, route_node_stringify},
    route_node::RouteNodeRc,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Router<'a> {
    root_node_rc: RouteNodeRc<'a>,
    leaf_nodes_rc: HashMap<&'a str, RouteNodeRc<'a>>,
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Self {
            root_node_rc: RouteNodeRc::default(),
            leaf_nodes_rc: HashMap::new(),
        }
    }

    pub fn insert_route(&mut self, name: &'a str, template: &'a str) -> &mut Self {
        let leaf_node_rc = route_node_insert(self.root_node_rc.clone(), name, template);
        self.leaf_nodes_rc.insert(name, leaf_node_rc);

        self
    }

    pub fn parse_route<'b>(&self, path: &'b str) -> (Option<&'a str>, HashMap<&'a str, &'b str>) {
        let (route_name, parameter_names, parameter_values) =
            route_node_parse(self.root_node_rc.clone(), path, 20);

        if let Some(route_name) = route_name {
            let parameters: HashMap<_, _> = parameter_names
                .clone()
                .into_iter()
                .zip(parameter_values)
                .collect();

            (Some(route_name), parameters)
        } else {
            Default::default()
        }
    }

    pub fn stringify_route<'b>(
        &self,
        route_name: &'a str,
        route_parameters: &HashMap<&'a str, &'b str>,
    ) -> Option<String> {
        if let Some(node_rc) = self.leaf_nodes_rc.get(route_name) {
            let mut parameter_values: Vec<&str> = Default::default();

            for parameter in node_rc.borrow().route_parameter_names.iter() {
                parameter_values.push(route_parameters.get(parameter).unwrap());
            }

            Some(route_node_stringify(node_rc.clone(), &parameter_values))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TEMPLATE_PLACEHOLDER_REGEX;
    use std::collections::HashSet;

    #[test]
    fn router_1() {
        let mut router = Router::new();
        router
            .insert_route("a", "/a")
            .insert_route("b", "/b/{x}")
            .insert_route("c", "/b/{y}/c")
            .insert_route("d", "/b/{z}/d");

        let (route_name, route_parameters) = router.parse_route("/a");
        assert_eq!(route_name.unwrap(), "a");
        assert_eq!(route_parameters, vec![].into_iter().collect(),);

        let (route_name, route_parameters) = router.parse_route("/b/123");
        assert_eq!(route_name.unwrap(), "b");
        assert_eq!(route_parameters, vec![("x", "123")].into_iter().collect(),);

        let (route_name, route_parameters) = router.parse_route("/b/456/c");
        assert_eq!(route_name.unwrap(), "c");
        assert_eq!(route_parameters, vec![("y", "456")].into_iter().collect(),);

        let (route_name, route_parameters) = router.parse_route("/b/789/d");
        assert_eq!(route_name.unwrap(), "d");
        assert_eq!(route_parameters, vec![("z", "789")].into_iter().collect(),);
    }

    #[test]
    fn router_2() {
        let mut router = Router::new();

        router.insert_route("aa", "a/{a}/a");
        router.insert_route("a", "a");

        router.insert_route("one", "/a");
        router.insert_route("two", "/a/{x}/{y}");
        router.insert_route("three", "/c/{x}");
        router.insert_route("four", "/c/{y}/{z}/");

        let (route_name, _route_parameters) = router.parse_route("/a");
        assert_eq!(route_name.unwrap(), "one");

        let (route_name, route_parameters) = router.parse_route("/a/1/2");
        assert_eq!(route_name.unwrap(), "two");
        assert_eq!(
            route_parameters,
            vec![("x", "1"), ("y", "2"),].into_iter().collect(),
        );

        let route_name = "two";
        let route_parameters = vec![("x", "1"), ("y", "2")].into_iter().collect();
        let path = router
            .stringify_route(route_name, &route_parameters)
            .unwrap();
        assert_eq!(path, "/a/1/2");

        let (route_name, route_parameters) = router.parse_route("/c/3");
        assert_eq!(route_name.unwrap(), "three");
        assert_eq!(route_parameters, vec![("x", "3"),].into_iter().collect(),);

        let (route_name, route_parameters) = router.parse_route("/c/3/4");
        assert_eq!(route_name.unwrap(), "three");
        assert_eq!(route_parameters, vec![("x", "3/4"),].into_iter().collect(),);

        let route_name = "three";
        let route_parameters = vec![("x", "3/4")].into_iter().collect();
        let path = router
            .stringify_route(route_name, &route_parameters)
            .unwrap();
        assert_eq!(path, "/c/3/4");

        let (route_name, route_parameters) = router.parse_route("/c/3/4/");
        assert_eq!(route_name.unwrap(), "four");
        assert_eq!(
            route_parameters,
            vec![("y", "3"), ("z", "4"),].into_iter().collect(),
        );
    }

    #[test]
    fn router_templates_small() {
        router_templates("small")
    }

    #[test]
    fn router_templates_docker() {
        router_templates("docker")
    }

    #[test]
    fn router_templates_github() {
        router_templates("github")
    }

    fn router_templates(name: &str) {
        let mut path = std::path::PathBuf::new();
        path.push("fixtures");
        path.push(name);
        path.set_extension("txt");

        let templates = std::fs::read_to_string(path.as_path()).unwrap();
        let templates: Vec<_> = templates
            .split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        let mut all_parameter_names: HashSet<&str> = Default::default();

        for template in templates.iter() {
            for captures in TEMPLATE_PLACEHOLDER_REGEX.captures_iter(template) {
                all_parameter_names.insert(captures.get(1).unwrap().as_str());
            }
        }

        let all_parameter_values: Vec<_> = (0..all_parameter_names.len())
            .map(|index| format!("p{}", index))
            .collect();

        let all_parameters: HashMap<_, _> = all_parameter_names
            .into_iter()
            .zip(all_parameter_values.iter())
            .map(|(name, value)| (name, value.as_str()))
            .collect();

        let mut router = Router::new();
        for template in templates.iter() {
            router.insert_route(template, template);
        }

        let paths: Vec<_> = templates
            .iter()
            .map(|template| router.stringify_route(template, &all_parameters).unwrap())
            .collect();

        for index in 0..templates.len() {
            let path = &paths[index];
            let template = templates[index];

            let (route_name, route_parameters) = router.parse_route(path.as_str());

            let expected_parameters: HashMap<_, _> = route_parameters
                .keys()
                .cloned()
                .map(|key| (key, all_parameters[key]))
                .collect();

            assert_eq!(route_name, Some(template));
            assert_eq!(route_parameters, expected_parameters);
        }
    }
}
