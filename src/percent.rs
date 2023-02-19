#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Percent(pub f32);

impl std::ops::Add for Percent {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for Percent {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::fmt::Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl Percent {
    pub fn clamp(self) -> Self {
        if self < Percent(0.0) {
            Percent(0.0)
        } else if self > Percent(100.0) {
            Percent(100.0)
        } else {
            self.clone()
        }
    }
}

fn f32_representation(s: &str) -> Result<f32, String> {
    let fs = s.trim_end_matches('%');
    let res: f32 = fs
        .parse()
        .map_err(|_| format!("Cannot parse {s} as 32-bit float."))?;
    Ok(res)
}

pub fn strictly_positive_percent(s: &str) -> Result<Percent, String> {
    let f = f32_representation(s)?;
    if f <= 0.0 || f > 100.0 {
        Err(format!(
            "'{f}' must be strictly positiive and no more than 100.0."
        ))
    } else {
        Ok(Percent(f))
    }
}

pub fn non_negative_percent(s: &str) -> Result<Percent, String> {
    let f = f32_representation(s)?;
    if f < 0.0 {
        Err(format!("'{f}' cannot be negative."))
    } else {
        Ok(Percent(f))
    }
}
