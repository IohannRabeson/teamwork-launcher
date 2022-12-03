use {
    nom::Finish,
    serde::{Deserialize, Serialize},
};
/// Text filter
/// 
/// This filter accepts multiple terms and search for text containing all the terms.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextFilter {
    /// Original input text.
    text: String,
    /// Terms parsed from the input text.
    terms: Vec<String>,
    /// Ignore the case when filtering
    ignore_case: bool,
    /// Ignore the case when filtering
    ignore_accents: bool,
}

impl TextFilter {
    pub fn new(text: &str) -> Self {
        let mut filter = Self {
            terms: Default::default(),
            text: String::new(),
            ignore_case: true,
            ignore_accents: true,
        };

        filter.set_text(text);

        filter
    }

    fn parse_terms(text: &str) -> Vec<String> {
        match parser::parse_terms(&text).finish() {
            Ok((_, terms)) => terms,
            Err(_) => Vec::new(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.terms = Self::parse_terms(text);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn accept(&self, text: &str) -> bool {
        let cleaned_input = self.clean_text(text);

        self.terms.is_empty() || self.terms.iter().all(|term| cleaned_input.contains(&self.clean_text(term)))
    }

    fn clean_text(&self, text: &str) -> String {
        use unidecode::unidecode;

        let mut result = text.to_string();

        if self.ignore_accents {
            result = unidecode(&result);
        }

        if self.ignore_case {
            result = result.to_lowercase();
        }

        result
    }
}

impl Default for TextFilter {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::TextFilter;

    #[test_case("hello", "", true; "accept all if no filter")]
    #[test_case("hello", "   ", true; "accept all if no filter (trimmed)")]
    #[test_case("É", "e", true; "ignore case and accents")]
    #[test_case("a", "b", false; "reject")]
    fn test_text_filter(input: &str, filter: &str, expect_accept: bool) {
        let text_filter = TextFilter::new(filter);

        assert_eq!(expect_accept, text_filter.accept(input))
    }
}

mod parser {
    use nom::{
        branch::alt,
        bytes::complete::is_not,
        character::complete::{char, multispace1},
        combinator::complete,
        multi::separated_list0,
        sequence::{delimited, preceded},
        IResult,
    };

    fn parse_term(s: &str) -> IResult<&str, &str> {
        let parse_quoted = delimited(char('\"'), is_not("\""), char('\"'));
        let parse_missing_quote = preceded(char('\"'), is_not("\""));
        let parse_word = is_not(" \t\r\n");

        alt((parse_quoted, parse_missing_quote, parse_word))(s)
    }

    pub(crate) fn parse_terms(s: &str) -> IResult<&str, Vec<String>> {
        complete(separated_list0(multispace1, parse_term))(s.trim())
            .map(|(remaining, terms)| (remaining, terms.iter().map(ToString::to_string).collect()))
    }

    #[cfg(test)]
    mod tests {
        use test_case::test_case;

        #[test_case("", &[]; "empty")]
        #[test_case("  ", &[]; "empty space only")]
        #[test_case("a", &["a"]; "unique")]
        #[test_case("a b", &["a", "b"]; "2 terms")]
        #[test_case("a \"b c\"", &["a", "b c"]; "2 terms second quoted")]
        #[test_case("a \"b c", &["a", "b c"]; "second quote missing")]
        #[test_case(" a ", &["a"]; "trim one")]
        #[test_case(" a b ", &["a", "b"]; "trim 2")]
        fn test_parse_terms(input: &str, expected_terms: &[&str]) {
            let (_, parsed_terms) = super::parse_terms(input).unwrap();

            assert_eq!(expected_terms, parsed_terms)
        }
    }
}
