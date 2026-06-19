use std::fmt;
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum Constant {
    Pi,
    E,
    Inf,
    True,
    False,
    Tau,
    Phi,
}

impl Constant {
    pub fn from(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "pi" => Some(Constant::Pi),
            "e" => Some(Constant::E),
            "inf" => Some(Constant::Inf),
            "true" => Some(Constant::True),
            "false" => Some(Constant::False),
            "tau" => Some(Constant::Tau),
            "phi" => Some(Constant::Phi),

            _ => None,
        }
    }

    pub fn value(&self) -> f64 {
        match self {
            Constant::Pi => std::f64::consts::PI,
            Constant::E => std::f64::consts::E,
            Constant::Inf => f64::INFINITY,
            Constant::True => 1.0,
            Constant::False => 0.0,
            Constant::Tau => std::f64::consts::TAU,
            Constant::Phi => 1.618_033_988_749_895,
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let constant = match self {
            Constant::Pi => "pi",
            Constant::E => "e",
            Constant::Inf => "inf",
            Constant::True => "true",
            Constant::False => "false",
            Constant::Tau => "tau",
            Constant::Phi => "phi",
        };

        write!(f, "{}", constant)
    }
}
