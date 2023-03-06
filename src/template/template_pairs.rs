use super::template_parts::{parse_template_parts, TemplateParts};
use regex::Regex;

pub fn parse_template_pairs<'a>(template: &'a str, re: &'a Regex) -> TemplatePairs<'a> {
    TemplatePairs::new(template, re)
}

pub struct TemplatePairs<'a> {
    parts: TemplateParts<'a>,
    index: usize,
    is_finished: bool,
}

impl<'a> TemplatePairs<'a> {
    fn new(template: &'a str, re: &'a Regex) -> Self {
        let parts = parse_template_parts(template, re);
        let index = 0;
        let is_finished = false;

        Self {
            parts,
            index,
            is_finished,
        }
    }
}

impl<'a> Iterator for TemplatePairs<'a> {
    type Item = (&'a str, Option<&'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        let result = if let Some(part) = self.parts.next() {
            if self.index == 0 {
                Some((part, None))
            } else {
                let anchor = self.parts.next().unwrap();
                Some((anchor, Some(part)))
            }
        } else {
            None
        };

        self.index += 1;

        if result.is_none() {
            self.is_finished = true;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::TEMPLATE_PLACEHOLDER_REGEX;
    use super::*;

    #[test]
    fn parse_template_pairs_test() {
        let pairs: Vec<_> =
            parse_template_pairs("/a/{b}/{c}", &TEMPLATE_PLACEHOLDER_REGEX).collect();

        assert_eq!(
            pairs,
            vec![("/a/", None), ("/", Some("b")), ("", Some("c"))]
        );

        let pairs: Vec<_> =
            parse_template_pairs("/a/{b}/{c}/", &TEMPLATE_PLACEHOLDER_REGEX).collect();

        assert_eq!(
            pairs,
            vec![("/a/", None), ("/", Some("b")), ("/", Some("c"))]
        );

        let pairs: Vec<_> = parse_template_pairs("", &TEMPLATE_PLACEHOLDER_REGEX).collect();

        assert_eq!(pairs, vec![("", None)])
    }
}
