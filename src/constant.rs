use std::f64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constant {
    PI,
    E,   // Euler's number
    Inf, // infinity
    True,
    False
}

impl Constant {
    pub fn from(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "pi" => Ok(Constant::PI),
            "e" => Ok(Constant::E),
            "inf" => Ok(Constant::Inf),
            "true" => Ok(Constant::True),
            "false" => Ok(Constant::False),
            _ => Err(format!("Invalid Constant: {}", name)),
        }
    }

    pub fn get_number(&self) -> f64 {
        match self {
            Constant::PI => f64::consts::PI,
            Constant::E => f64::consts::E,
            Constant::Inf => f64::INFINITY,

            Constant::True => 1.0,
            Constant::False => 0.0,
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Constant::PI => "PI",
            Constant::E => "e",
            Constant::Inf => "Inf",
            Constant::True => "true",
            Constant::False => "false",
        };

        write!(f, "{}", name)
    }
}
