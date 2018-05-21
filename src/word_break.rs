/// Simple word breaking iterator for caching purposes
pub fn simple(text: &str) -> Simple {
    Simple { text, pos: 0 }
}

pub struct Simple<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Iterator for Simple<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;
        let c = self.text[start..].chars().next()?;
        self.pos += c.len_utf8();

        if break_after(c) {
            return Some(&self.text[start..self.pos])
        }

        for (i, c) in self.text[self.pos..].char_indices() {
            if break_before(c) {
                self.pos += i;
                return Some(&self.text[start..self.pos])
            }
        }

        self.pos = self.text.len();
        return Some(&self.text[start..self.pos])
    }
}

fn is_word_space(c: char) -> bool {
    c == ' ' || c == '\u{a0}'
}

fn break_after(c: char) -> bool {
    is_word_space(c) || (c >= '\u{2000}' && c <= '\u{200a}') || c == '\u{3000}'
}

fn break_before(c: char) -> bool {
    // CJK ideographs (and yijing hexagram symbols)
    break_after(c) || (c >= '\u{3400}' && c <= '\u{9fff}')
}

#[test]
fn test() {
    assert!(simple("hello world").eq(vec!["hello", " ", "world"]));
}
