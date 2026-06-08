use std::{f64, fmt};

use crate::err_fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
    // 1-arg functions
    Sin,
    Cos,
    Tan,
    Sqrt,
    Abs,
    Ln,
    Exp,
    Floor,
    Ceil,
    Round,
    Recip,
    Cbrt,
    Log,
    Rad,
    Deg,
    Pow,

    Trunc, // truncate decimals
    Int,
    Asin,
    Acos,
    Atan,
    Atan2,
    Sinh,
    Cosh,
    Tanh,
    Clamp,
    Gcd, // greatest common denominator
    Lcm, // least common multiple

    Max,
    Min,
}

impl Function {
    pub fn from(name: &str) -> Option<Self> {
        use Function as F;
        match name {
            "sin" => Some(F::Sin),
            "cos" => Some(F::Cos),
            "tan" => Some(F::Tan),
            "sqrt" => Some(F::Sqrt),
            "abs" => Some(F::Abs),
            "ln" => Some(F::Ln),
            "exp" => Some(F::Exp),
            "floor" => Some(F::Floor),
            "ceil" => Some(F::Ceil),
            "round" => Some(F::Round),
            "recip" => Some(F::Recip),
            "cbrt" => Some(F::Cbrt),
            "log" => Some(F::Log),
            "max" => Some(F::Max),
            "min" => Some(F::Min),
            "pow" => Some(F::Pow),
            "deg" => Some(F::Deg),
            "rad" => Some(F::Rad),
            "int" => Some(F::Int),
            "trunc" => Some(F::Trunc),
            "asin" => Some(F::Asin),
            "acos" => Some(F::Acos),
            "atan" => Some(F::Atan),
            "sinh" => Some(F::Sinh),
            "cosh" => Some(F::Cosh),
            "tanh" => Some(F::Tanh),
            "clamp" => Some(F::Clamp),
            "lcm" => Some(F::Lcm),
            "gcd" => Some(F::Gcd),
            "atan2" => Some(F::Atan2),
            _ => None,
        }
    }

    // Returns the (min, max) accepted argument count for this function.
    pub fn arity(&self) -> (usize, usize) {
        use Function as F;

        match self {
            F::Max | F::Min => (2, usize::MAX),
            F::Sqrt | F::Log => (1, 2),
            F::Pow | F::Gcd | F::Atan2 | F::Lcm => (2, 2),
            F::Clamp => (3, 3),

            _ => (1, 1),
        }
    }

    pub fn call(&self, args: &[f64]) -> Result<f64, String> {
        use Function as F;

        let (min, max) = self.arity();
        if !(min..=max).contains(&args.len()) {
            // special case for usize::MAX or min==max
            let arity_str = if max == usize::MAX {
                format!("at least {}", min)
            } else if min == max {
                format!("{}", min)
            } else {
                format!("{}-{}", min, max)
            };

            return err_fmt!(
                "Eval Error: function {} takes {} argument(s) but got {}",
                self,
                arity_str,
                args.len()
            );
        }

        let result = match self {
            F::Int | F::Trunc => args[0].trunc(),
            F::Asin => args[0].asin(),
            F::Acos => args[0].acos(),
            F::Atan => args[0].atan(),
            F::Atan2 => args[0].atan2(args[1]),
            F::Sinh => args[0].sinh(),
            F::Cosh => args[0].cosh(),
            F::Tanh => args[0].tanh(),
            F::Sin => args[0].sin(),
            F::Cos => args[0].cos(),
            F::Clamp => {
                //clamp(x, min, max). min <= max.
                let x = args[0];
                let min = args[1];
                let max = args[2];

                if min > max {
                    return err_fmt!(
                        "Eval Error: clamp range min ({}) must be less than max ({})",
                        min,
                        max
                    );
                }

                x.clamp(min, max)
            }
            F::Lcm => lcm(args[0], args[1]),
            F::Gcd => gcd(args[0], args[1]),

            F::Tan => {
                let normalized = (args[0] / f64::consts::FRAC_PI_2).round();
                if (args[0] - normalized * f64::consts::FRAC_PI_2).abs() < f64::EPSILON
                    && normalized as i64 % 2 != 0
                {
                    return err_fmt!(
                        "Eval Error: function tan({}) is undefined at asymptote (pi / 2 + n * pi)",
                        args[0]
                    );
                }

                args[0].tan()
            }
            F::Abs => args[0].abs(),
            F::Ln => args[0].ln(),
            F::Exp => args[0].exp(),
            F::Floor => args[0].floor(),
            F::Ceil => args[0].ceil(),
            F::Round => args[0].round(),
            F::Cbrt => args[0].cbrt(),
            F::Recip => {
                if args[0] == 0f64 {
                    return Err("Eval Error: recip(0) is undefined (division by zero)".to_string());
                }

                args[0].recip()
            }
            F::Sqrt => {
                let root = if args.len() == 2 { args[1] } else { 2.0 };
                let x = args[0];

                if root == 0f64 {
                    return err_fmt!(
                        "Eval Error: sqrt({0}, 0) is undefined (division by zero: {0} ^ (1/0)",
                        x
                    );
                }

                if x < 0.0 && root.fract() == 0.0 && (root as i64) % 2 != 0 {
                    // odd integer root of neg numbers
                    -(-x).powf(1.0 / root)
                } else {
                    x.powf(1.0 / root)
                }
            }
            F::Log => {
                let base = if args.len() == 2 { args[1] } else { 10.0 };

                args[0].log(base)
            }

            F::Max => args.iter().copied().reduce(f64::max).unwrap(),
            F::Min => args.iter().copied().reduce(f64::min).unwrap(),

            F::Pow => {
                if args[0] == 0f64 && args[1] < 0f64 {
                    return err_fmt!(
                        "Eval Error: pow({0}, {1}) is undefined (division by zero: 1/{0}^{2})",
                        args[0],
                        args[1],
                        args[1].abs()
                    );
                }

                args[0].powf(args[1])
            }
            F::Rad => args[0].to_radians(),
            F::Deg => args[0].to_degrees(),
        };

        if result.is_nan() {
            return err_fmt!(
                "Eval Error: {}({}) is undefined",
                self,
                args.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }

        Ok(result)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Function as F;

        let name = match self {
            F::Sin => "sin",
            F::Cos => "cos",
            F::Tan => "tan",
            F::Sqrt => "sqrt",
            F::Abs => "abs",
            F::Ln => "ln",
            F::Exp => "exp",
            F::Floor => "floor",
            F::Ceil => "ceil",
            F::Round => "round",
            F::Recip => "recip",
            F::Cbrt => "cbrt",
            F::Log => "log",
            F::Max => "max",
            F::Min => "min",
            F::Pow => "pow",
            F::Rad => "rad",
            F::Deg => "deg",
            F::Int => "int",
            F::Trunc => "trunc",
            F::Asin => "asin",
            F::Acos => "acos",
            F::Atan => "atan",
            F::Sinh => "sinh",
            F::Cosh => "cosh",
            F::Tanh => "tanh",
            F::Clamp => "clamp",
            F::Gcd => "GCD",
            F::Lcm => "LCM",
            F::Atan2 => "atan2",
        };

        write!(f, "{}", name)
    }
}

fn gcd(mut a: f64, mut b: f64) -> f64 {
    if a == b {
        return a;
    }

    if b > a {
        std::mem::swap(&mut a, &mut b);
    }

    while b > 0f64 {
        let temp = a;
        a = b;
        b = temp % b;
    }

    a
}

fn lcm(a: f64, b: f64) -> f64 {
    a * (b / gcd(a, b))
}
