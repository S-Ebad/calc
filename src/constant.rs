use std::f64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constant {
    PI,
    E,   // Euler's number
    Inf, // infinity
}

impl Constant {
    pub fn from(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "pi" => Ok(Constant::PI),
            "e" => Ok(Constant::E),
            "inf" => Ok(Constant::Inf),
            _ => Err(format!("Invalid Constant: {}", name)),
        }
    }

    pub fn get_number(&self) -> f64 {
        match self {
            Constant::PI => f64::consts::PI,
            Constant::E => f64::consts::E,
            Constant::Inf => f64::INFINITY,
        }
    }
}
