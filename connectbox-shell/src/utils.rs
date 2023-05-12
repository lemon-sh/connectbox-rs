pub(crate) struct QuotableArgs<'a> {
    s: &'a str,
}

impl<'a> QuotableArgs<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s }
    }
}

impl<'a> Iterator for QuotableArgs<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.s = self.s.trim_start();
        if self.s.is_empty() {
            return None;
        }
        if self.s.as_bytes()[0] == b'"' {
            self.s = &self.s[1..];
            if let Some(pos) = self.s.find('"') {
                let result = &self.s[..pos];
                self.s = &self.s[pos + 1..];
                return Some(result);
            }
            let result = self.s;
            self.s = &self.s[..0];
            return Some(result);
        }
        if let Some(pos) = self.s.find(char::is_whitespace) {
            let result = &self.s[..pos];
            self.s = &self.s[pos..];
            Some(result)
        } else {
            let result = self.s;
            self.s = &self.s[..0];
            Some(result)
        }
    }
}
