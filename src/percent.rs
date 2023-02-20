#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Percent(pub u8);

impl std::ops::Add for Percent {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if u64::from(self.0) + u64::from(other.0) > 100 {
            Self(100)
        } else {
            Self(self.0 + other.0)
        }
    }
}

impl std::ops::Sub for Percent {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if self.0 < other.0 {
            Self(0)
        } else {
            Self(self.0 - other.0)
        }
    }
}

impl std::fmt::Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl std::convert::From<f32> for Percent {
    fn from(val: f32) -> Self {
        if val >= 100.0 {
            Percent(100)
        } else if val >= 0.0 {
            Percent(val.round() as u8)
        } else {
            Percent(0)
        }
    }
}

impl Percent {
    pub fn decimal(self) -> f32 {
        f32::from(self.0) / 100.0
    }
    fn u8_representation(s: &str) -> Result<u8, String> {
        let fs = s.trim_end_matches('%');
        let res: u8 = fs
            .parse()
            .map_err(|_| format!("Cannot parse {s} as 8-bit integer."))?;
        Ok(res)
    }
    pub fn strictly_positive(s: &str) -> Result<Percent, String> {
        let u = Self::u8_representation(s)?;
        if u == 0 || u > 100 {
            Err(format!(
                "'{u}' must be strictly positiive and no more than 100."
            ))
        } else {
            Ok(Percent(u))
        }
    }
    pub fn valid(s: &str) -> Result<Percent, String> {
        let u = Self::u8_representation(s)?;
        if u > 100 {
            Err(format!("'{u}' cannot be more than 100."))
        } else {
            Ok(Percent(u))
        }
    }
}
