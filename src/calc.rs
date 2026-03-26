use std::collections::HashMap;

pub struct Calculator {
  pub variables: HashMap<String, f64>,
}

impl Calculator {
  pub fn new() -> Self {
    Calculator {
      variables: HashMap::new(),
    }
  }

  pub fn add_variable(&mut self, name: &str, value: f64) {
    self.variables.insert(name.to_string(), value);
  }

  pub fn solve(&mut self, buf: &str) -> Result<f64, String> {
    let result = Self::tokenize(buf)
      .and_then(|tokens| self.parse(tokens))
      .and_then(|tokens| self.eval(tokens));

    if let Ok(ans) = result {
      self.add_variable("ans", ans);
    }

    result
  }
}
