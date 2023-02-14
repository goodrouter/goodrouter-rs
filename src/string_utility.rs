use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::cmp;

pub fn find_common_prefix_length(chars_left: &Vec<char>, chars_right: &Vec<char>) -> usize {
    let common_length = cmp::min(chars_left.len(), chars_right.len());

    let mut index = 0;
    while index < common_length {
        let char_left = chars_left[index];
        let char_right = chars_right[index];
        if char_left != char_right {
            break;
        }

        index += 1;
    }

    return index;
}

pub static PLACEHOLDER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{(.*?)\}").unwrap());

pub fn parse_placeholders<'a, 'b>(template: &'a str, re: &'b Regex) -> PlaceholderParts<'a> {
    PlaceholderParts::new(template, re)
}

pub struct PlaceholderParts<'a> {
    template: &'a str,
    matches: Vec<Captures<'a>>,
    state: PlaceholderPartsState,
}

enum PlaceholderPartsState {
    Part(usize, usize),
    Finished,
}

impl<'a> PlaceholderParts<'a> {
    fn new<'b>(template: &'a str, re: &'b Regex) -> Self {
        let matches: Vec<_> = re.captures_iter(template).collect();
        let state = PlaceholderPartsState::Part(0, 0);

        Self {
            template,
            matches,
            state,
        }
    }
}

impl<'a> Iterator for PlaceholderParts<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            PlaceholderPartsState::Part(part_index, part_offset)
                if part_index / 2 < self.matches.len() && part_index % 2 == 0 =>
            {
                let current_match = &self.matches[part_index / 2];
                let first_capture = current_match.get(0).unwrap();

                let part_index_next = part_index + 1;
                let part_offset_next = first_capture.start();
                self.state = PlaceholderPartsState::Part(part_index_next, part_offset_next);

                return Some(&self.template[part_offset..part_offset_next]);
            }

            PlaceholderPartsState::Part(part_index, _)
                if part_index / 2 < self.matches.len() && part_index % 2 == 1 =>
            {
                let current_match = &self.matches[part_index / 2];
                let first_capture = current_match.get(0).unwrap();
                let current_capture = current_match.get(1).unwrap();

                let part_index_next = part_index + 1;
                let part_offset_next = first_capture.end();
                self.state = PlaceholderPartsState::Part(part_index_next, part_offset_next);

                return Some(current_capture.as_str());
            }

            PlaceholderPartsState::Part(_, part_offset) => {
                self.state = PlaceholderPartsState::Finished;

                return Some(&self.template[part_offset..]);
            }

            PlaceholderPartsState::Finished => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_prefix_length_test() {
        assert_eq!(
            find_common_prefix_length(
                &String::from("ab").chars().collect(),
                &String::from("abc").chars().collect()
            ),
            2
        );

        assert_eq!(
            find_common_prefix_length(
                &String::from("abc").chars().collect(),
                &String::from("abc").chars().collect()
            ),
            3
        );

        assert_eq!(
            find_common_prefix_length(
                &String::from("bc").chars().collect(),
                &String::from("abc").chars().collect()
            ),
            0,
        );
    }

    #[test]
    fn parse_placeholders_test() {
        let parts: Vec<_> = parse_placeholders("/a/{b}/{c}", &PLACEHOLDER_REGEX).collect();

        assert_eq!(parts, vec!["/a/", "b", "/", "c", ""]);

        let parts: Vec<_> = parse_placeholders("/a/{b}/{c}/", &PLACEHOLDER_REGEX).collect();

        assert_eq!(parts, vec!["/a/", "b", "/", "c", "/"]);

        let parts: Vec<_> = parse_placeholders("", &PLACEHOLDER_REGEX).collect();

        assert_eq!(parts, vec![""])
    }
}
