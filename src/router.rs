use crate::{
    route_node::route_node_rc::{route_node_insert, route_node_parse, route_node_stringify},
    route_node::RouteNodeRc,
    template::TEMPLATE_PLACEHOLDER_REGEX,
};
use regex::Regex;
use std::{borrow::Cow, collections::HashMap};

type ParameterValueEncoder = dyn Fn(&str) -> Cow<str>;
type ParameterValueDecoder = dyn Fn(&str) -> Cow<str>;

pub struct Router<'a> {
    root_node_rc: RouteNodeRc<'a>,
    leaf_nodes_rc: HashMap<&'a str, RouteNodeRc<'a>>,
    maximum_parameter_value_length: usize,
    parameter_placeholder_re: &'a Regex,
    parameter_value_encoder: Box<ParameterValueEncoder>,
    parameter_value_decoder: Box<ParameterValueDecoder>,
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        fn parameter_encoder(value: &str) -> Cow<str> {
            urlencoding::encode(value)
        }
        fn parameter_decoder(value: &str) -> Cow<str> {
            urlencoding::decode(value).unwrap_or(Cow::Borrowed(value))
        }

        let parameter_value_encoder = Box::new(parameter_encoder);
        let parameter_value_decoder = Box::new(parameter_decoder);

        Self {
            root_node_rc: RouteNodeRc::default(),
            leaf_nodes_rc: HashMap::new(),
            maximum_parameter_value_length: 20,
            parameter_placeholder_re: &TEMPLATE_PLACEHOLDER_REGEX,
            parameter_value_encoder,
            parameter_value_decoder,
        }
    }

    pub fn set_maximum_parameter_value_length(&mut self, value: usize) -> &mut Self {
        self.maximum_parameter_value_length = value;

        self
    }

    pub fn set_parameter_placeholder_re(&mut self, value: &'a Regex) -> &mut Self {
        self.parameter_placeholder_re = value;

        self
    }

    pub fn set_parameter_value_encoder(&mut self, value: Box<ParameterValueEncoder>) -> &mut Self {
        self.parameter_value_encoder = value;

        self
    }

    pub fn set_parameter_value_decoder(&mut self, value: Box<ParameterValueDecoder>) -> &mut Self {
        self.parameter_value_decoder = value;

        self
    }

    pub fn insert_route(&mut self, name: &'a str, template: &'a str) -> &mut Self {
        let leaf_node_rc = route_node_insert(
            self.root_node_rc.clone(),
            name,
            template,
            self.parameter_placeholder_re,
        );
        self.leaf_nodes_rc.insert(name, leaf_node_rc);

        self
    }

    pub fn parse_route<'b>(
        &self,
        path: &'b str,
    ) -> (Option<&'a str>, HashMap<&'a str, Cow<'b, str>>) {
        let (route_name, parameter_names, parameter_values) = route_node_parse(
            self.root_node_rc.clone(),
            path,
            self.maximum_parameter_value_length,
        );

        if let Some(route_name) = route_name {
            let parameters: HashMap<_, _> = parameter_names
                .clone()
                .into_iter()
                .zip(
                    parameter_values
                        .iter()
                        .map(|parameter_value| (self.parameter_value_decoder)(parameter_value)),
                )
                .collect();

            (Some(route_name), parameters)
        } else {
            Default::default()
        }
    }

    pub fn stringify_route(
        &self,
        route_name: &'a str,
        route_parameters: &'a HashMap<&'a str, &'a str>,
    ) -> Option<Cow<str>> {
        if let Some(node_rc) = self.leaf_nodes_rc.get(route_name) {
            let parameter_values: Vec<_> = node_rc
                .borrow()
                .route_parameter_names
                .iter()
                .map(|parameter_name| route_parameters.get(parameter_name).unwrap())
                .map(|parameter_value| (self.parameter_value_encoder)(parameter_value))
                .collect();

            Some(route_node_stringify(node_rc.clone(), parameter_values))
        } else {
            None
        }
    }
}

impl<'a> Default for Router<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(
            route_parameters,
            vec![("x", "123")]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
        );

        let (route_name, route_parameters) = router.parse_route("/b/456/c");
        assert_eq!(route_name.unwrap(), "c");
        assert_eq!(
            route_parameters,
            vec![("y", "456")]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
        );

        let (route_name, route_parameters) = router.parse_route("/b/789/d");
        assert_eq!(route_name.unwrap(), "d");
        assert_eq!(
            route_parameters,
            vec![("z", "789")]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
        );
    }

    #[test]
    fn router_2() {
        let mut router = Router::new();

        router
            .insert_route("aa", "a/{a}/a")
            .insert_route("a", "a")
            .insert_route("one", "/a")
            .insert_route("two", "/a/{x}/{y}")
            .insert_route("three", "/c/{x}")
            .insert_route("four", "/c/{y}/{z}/");

        let (route_name, _route_parameters) = router.parse_route("/a");
        assert_eq!(route_name.unwrap(), "one");

        let (route_name, route_parameters) = router.parse_route("/a/1/2");
        assert_eq!(route_name.unwrap(), "two");
        assert_eq!(
            route_parameters,
            vec![("x", "1"), ("y", "2"),]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
        );

        let route_name = "two";
        let route_parameters = vec![("x", "1"), ("y", "2")].into_iter().collect();
        let path = router
            .stringify_route(route_name, &route_parameters)
            .unwrap();
        assert_eq!(path, "/a/1/2");

        let (route_name, route_parameters) = router.parse_route("/c/3");
        assert_eq!(route_name.unwrap(), "three");
        assert_eq!(
            route_parameters,
            vec![("x", "3"),]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
        );

        let (route_name, route_parameters) = router.parse_route("/c/3/4");
        assert_eq!(route_name.unwrap(), "three");
        assert_eq!(
            route_parameters,
            vec![("x", "3/4"),]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
        );

        let route_name = "three";
        let route_parameters = vec![("x", "3/4")].into_iter().collect();
        let path = router
            .stringify_route(route_name, &route_parameters)
            .unwrap();
        assert_eq!(path, "/c/3%2F4");

        let (route_name, route_parameters) = router.parse_route("/c/3/4/");
        assert_eq!(route_name.unwrap(), "four");
        assert_eq!(
            route_parameters,
            vec![("y", "3"), ("z", "4"),]
                .into_iter()
                .map(|(k, v)| (k, Cow::Borrowed(v)))
                .collect(),
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
            .zip(all_parameter_values.iter().map(|v| v.as_str()))
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

            let (route_name, route_parameters) = router.parse_route(path);

            let expected_parameters: HashMap<_, _> = route_parameters
                .keys()
                .cloned()
                .map(|key| (key, Cow::Borrowed(all_parameters[key])))
                .collect();

            assert_eq!(route_name, Some(template));
            assert_eq!(route_parameters, expected_parameters);
        }
    }
}
