use super::route_node_merge::*;
use super::*;
use crate::template::template_pairs::parse_template_pairs;
use crate::template::TEMPLATE_PLACEHOLDER_REGEX;
use std::cmp::min;

pub fn route_node_parse<'a, 'b>(
    node_rc: RouteNodeRc<'a>,
    path: &'b str,
    maximum_parameter_value_length: usize,
) -> (Option<&'a str>, Vec<&'a str>, Vec<&'b str>) {
    let mut path = path;
    let mut parameter_values: Vec<&str> = Default::default();

    let node = node_rc.borrow();

    if node.has_parameter {
        // we are matching a parameter value! If the path's length is 0, there is no match, because a parameter value should have at least length 1
        if path.is_empty() {
            return Default::default();
        }

        // look for the anchor in the path. If the anchor is empty, match the remainder of the path
        let index = if node.anchor.is_empty() {
            Some(path.len())
        } else {
            path[..min(
                node.anchor.len() + maximum_parameter_value_length,
                path.len(),
            )]
                .find(node.anchor)
        };

        if let Some(index) = index {
            let value = &path[..index];

            // remove the matches part from the path
            path = &path[index + node.anchor.len()..];

            parameter_values.push(value);
        } else {
            return Default::default();
        }
    } else {
        // if this node does not represent a parameter we expect the path to start with the `anchor`
        if !path.starts_with(node.anchor) {
            // this node does not match the path
            return Default::default();
        }

        // we successfully matches the node to the path, now remove the matched part from the path
        path = &path[node.anchor.len()..];
    }

    for child_rc in &node.children {
        if let (Some(child_route_name), child_route_parameter_names, mut child_parameters_values) =
            route_node_parse(child_rc.clone(), path, maximum_parameter_value_length)
        {
            let mut parameters = parameter_values.clone();
            parameters.append(&mut child_parameters_values);
            return (
                Some(child_route_name),
                child_route_parameter_names,
                parameters,
            );
        }
    }

    // if the node had a route name and there is no path left to match against then we found a route
    if path.is_empty() {
        if let Some(route_name) = node.route_name {
            return (
                Some(route_name),
                node.route_parameter_names.clone(),
                parameter_values,
            );
        }
    }

    Default::default()
}

pub fn route_node_stringify(node_rc: RouteNodeRc, parameter_values: &Vec<&str>) -> String {
    let mut parameter_index = parameter_values.len();
    let mut path_parts: Vec<&str> = Vec::new();
    let mut current_node_rc = Some(node_rc);

    while let Some(node_rc) = current_node_rc {
        let node = node_rc.borrow();
        path_parts.insert(0, node.anchor);

        if node.has_parameter {
            parameter_index -= 1;
            let value = parameter_values[parameter_index];
            path_parts.insert(0, value);
        }

        current_node_rc = node
            .parent
            .as_ref()
            .map(|parent_node_weak| parent_node_weak.upgrade().unwrap());
    }

    path_parts.join("")
}

pub fn route_node_insert<'a>(
    root_node_rc: RouteNodeRc<'a>,
    name: &'a str,
    template: &'a str,
) -> RouteNodeRc<'a> {
    let template_pairs: Vec<_> =
        parse_template_pairs(template, &TEMPLATE_PLACEHOLDER_REGEX).collect();
    let route_parameter_names: Vec<_> = template_pairs
        .clone()
        .into_iter()
        .filter_map(|(_anchor, parameter)| parameter)
        .collect();

    let mut node_current_rc = root_node_rc.clone();
    for index in 0..template_pairs.len() {
        let (anchor, parameter) = template_pairs[index];
        let has_parameter = parameter.is_some();
        let route_name = if index == template_pairs.len() - 1 {
            Some(name)
        } else {
            None
        };

        let (common_prefix_length, child_node_rc) =
            route_node_find_similar_child(&node_current_rc.borrow(), anchor, has_parameter);

        node_current_rc = route_node_merge(
            node_current_rc,
            child_node_rc,
            anchor,
            has_parameter,
            route_name,
            route_parameter_names.clone(),
            common_prefix_length,
        );
    }

    node_current_rc
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn route_node_permutations() {
        let route_configs = vec!["/a", "/b/{x}", "/b/{x}/", "/b/{x}/c", "/b/{y}/d"];

        let mut node_root_previous_rc = None;

        for route_configs in route_configs.iter().permutations(route_configs.len()) {
            let node_root_rc = Rc::new(RefCell::new(RouteNode::default()));

            for template in route_configs {
                route_node_insert(node_root_rc.clone(), template, template);
            }

            {
                let node_root = node_root_rc.borrow();
                assert_eq!(node_root.children.len(), 1);
            }

            if let Some(node_root_previous) = node_root_previous_rc {
                assert_eq!(node_root_rc, node_root_previous);
            }

            node_root_previous_rc = Some(node_root_rc.clone());
        }
    }
}
