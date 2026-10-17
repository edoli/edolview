//! Small, dependency-free arithmetic expression parser for numeric UI fields.

/// Evaluates a finite arithmetic expression.
///
/// Supported operators are `+`, `-`, `*`, `/`, `%`, and `^`; parentheses and
/// the `pi`/`π` and `e` constants are also accepted.
pub fn parse_number_expression(input: &str) -> Option<f64> {
    let mut parser = Parser::new(input);
    let value = parser.parse_expression()?;
    parser.skip_whitespace();
    (parser.is_eof() && value.is_finite()).then_some(value)
}

pub fn parse_i32_expression(input: &str) -> Option<i32> {
    let value = parse_number_expression(input)?;
    let rounded = value.round();
    ((value - rounded).abs() <= f64::EPSILON && rounded >= i32::MIN as f64 && rounded <= i32::MAX as f64)
        .then_some(rounded as i32)
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }

    fn parse_expression(&mut self) -> Option<f64> {
        let mut value = self.parse_term()?;
        loop {
            if self.consume('+') {
                value += self.parse_term()?;
            } else if self.consume('-') {
                value -= self.parse_term()?;
            } else {
                return Some(value);
            }
        }
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut value = self.parse_power()?;
        loop {
            if self.consume('*') {
                value *= self.parse_power()?;
            } else if self.consume('/') {
                value /= self.parse_power()?;
            } else if self.consume('%') {
                value %= self.parse_power()?;
            } else {
                return Some(value);
            }
        }
    }

    fn parse_power(&mut self) -> Option<f64> {
        let value = self.parse_unary()?;
        if self.consume('^') {
            Some(value.powf(self.parse_power()?))
        } else {
            Some(value)
        }
    }

    fn parse_unary(&mut self) -> Option<f64> {
        if self.consume('+') {
            self.parse_unary()
        } else if self.consume('-') {
            Some(-self.parse_unary()?)
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Option<f64> {
        if self.consume('(') {
            let value = self.parse_expression()?;
            self.consume(')').then_some(value)
        } else if self.consume_keyword("pi") || self.consume_keyword("π") {
            Some(std::f64::consts::PI)
        } else if self.consume_keyword("e") {
            Some(std::f64::consts::E)
        } else {
            self.parse_number()
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let start = self.offset;
        let mut saw_digit = false;

        while matches!(self.peek_char(), Some('0'..='9')) {
            saw_digit = true;
            self.advance_char();
        }
        if self.peek_char() == Some('.') {
            self.advance_char();
            while matches!(self.peek_char(), Some('0'..='9')) {
                saw_digit = true;
                self.advance_char();
            }
        }
        if !saw_digit {
            return None;
        }
        if matches!(self.peek_char(), Some('e' | 'E')) {
            let exponent_start = self.offset;
            self.advance_char();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.advance_char();
            }
            let digits_start = self.offset;
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.advance_char();
            }
            if digits_start == self.offset {
                self.offset = exponent_start;
            }
        }

        self.input[start..self.offset].trim().parse().ok()
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let remaining = &self.input[self.offset..];
        if remaining
            .get(..keyword.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
        {
            self.offset += keyword.len();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        self.skip_whitespace();
        if self.peek_char() == Some(expected) {
            self.advance_char();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
            self.advance_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn advance_char(&mut self) {
        if let Some(character) = self.peek_char() {
            self.offset += character.len_utf8();
        }
    }

    fn is_eof(&self) -> bool {
        self.offset == self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_i32_expression, parse_number_expression};

    #[test]
    fn evaluates_arithmetic_and_constants() {
        assert_eq!(parse_number_expression("4 * 2"), Some(8.0));
        assert!((parse_number_expression("2 + pi").unwrap() - 5.141_592_653_589_793).abs() < 1e-12);
        assert_eq!(parse_number_expression("(2 + 3) * 4"), Some(20.0));
    }

    #[test]
    fn only_accepts_integral_values_for_i32() {
        assert_eq!(parse_i32_expression("1920 / 2"), Some(960));
        assert_eq!(parse_i32_expression("pi"), None);
    }

    #[test]
    fn rejects_unsupported_unicode_without_panicking() {
        for input in ["한", "한글", "ㄱ", "🙂", "é", "p🙂", "1 + 한글", "(🙂)", "pi한", "e🙂"] {
            assert_eq!(parse_number_expression(input), None, "{input}");
            assert_eq!(parse_i32_expression(input), None, "{input}");
        }
        assert_eq!(parse_number_expression("π"), Some(std::f64::consts::PI));
    }
}
