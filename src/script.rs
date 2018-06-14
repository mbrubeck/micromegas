use unicode_script::{get_script, Script};

pub fn script_runs(text: &str) -> ScriptRuns {
    ScriptRuns { text, script: Script::Unknown, pos: 0 }
}

pub struct ScriptRuns<'a> {
    text: &'a str,
    script: Script,
    pos: usize,
}

impl<'a> Iterator for ScriptRuns<'a> {
    type Item = (Script, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;
        for (i, c) in self.text[self.pos..].char_indices() {
            let script = get_script(c);
            if script != self.script {
                match self.script {
                    Script::Unknown | Script::Inherited | Script::Common => {
                        self.script = script;
                        continue;
                    }
                    _ => {}
                }
                match script {
                    Script::Inherited | Script::Common => continue,
                    _ => {}
                }
                self.pos = i;
                self.script = script;
                return Some((script, &self.text[start..i]));
            }
        }
        if self.pos < self.text.len() {
            self.pos = self.text.len();
            return Some((self.script, &self.text[start..]))
        }
        None
    }
}
