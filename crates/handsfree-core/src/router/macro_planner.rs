use regex::Regex;

pub struct MacroPlanner {
    split_regex: Regex,
    max_steps: usize,
}

impl Default for MacroPlanner {
    fn default() -> Self {
        Self::new(5)
    }
}

impl MacroPlanner {
    pub fn new(max_steps: usize) -> Self {
        let split_regex = Regex::new(r"(?i)\b(?:and then|after that|then|next)\b").unwrap();
        Self {
            split_regex,
            max_steps,
        }
    }

    /// Split a composite command into sequential sub-commands.
    pub fn split_macro(&self, input: &str) -> Vec<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let parts: Vec<String> = self
            .split_regex
            .split(trimmed)
            .map(|s| {
                s.trim_matches(|c: char| c == ',' || c.is_whitespace())
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            vec![trimmed.to_string()]
        } else {
            parts.into_iter().take(self.max_steps).collect()
        }
    }
}
