use regex::{Captures, Regex};

pub fn parse_template_parts(template: &str) -> TemplateParts {
    TemplateParts::new(template)
}

pub struct TemplateParts<'a> {
    template: &'a str,
    matches: Vec<Captures<'a>>,
    state: TemplatePartsState,
}

enum TemplatePartsState {
    Part(usize, usize),
    Finished,
}

impl<'a> TemplateParts<'a> {
    fn new(template: &'a str) -> Self {
        let re = Regex::new(r"(.*?)\{(.*?)\}").unwrap();

        let matches: Vec<_> = re.captures_iter(template).collect();
        let state = TemplatePartsState::Part(0, 0);

        Self {
            template,
            matches,
            state,
        }
    }
}

impl<'a> Iterator for TemplateParts<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            TemplatePartsState::Part(part_index, _) if part_index / 2 < self.matches.len() => {
                let current_match = &self.matches[part_index / 2];
                let first_capture = current_match.get(0).unwrap();
                let current_capture = current_match.get(part_index % 2 + 1).unwrap();

                self.state = TemplatePartsState::Part(part_index + 1, first_capture.end());

                return Some(current_capture.as_str());
            }

            TemplatePartsState::Part(_, part_offset) => {
                self.state = TemplatePartsState::Finished;

                return Some(&self.template[part_offset..]);
            }

            TemplatePartsState::Finished => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_parts() {
        let parts: Vec<_> = parse_template_parts("/a/{b}/{c}").collect();

        assert_eq!(parts, vec!["/a/", "b", "/", "c", ""])
    }

    #[test]
    fn template_parts_empty() {
        let parts: Vec<_> = parse_template_parts("").collect();

        assert_eq!(parts, vec![""])
    }
}
