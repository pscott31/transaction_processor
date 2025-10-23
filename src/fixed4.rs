use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed4(i64);

impl Fixed4 {
    const SCALE: i64 = 10_000;
}

impl Fixed4 {
    pub fn zero() -> Self {
        Self(0)
    }
}

impl FromStr for Fixed4 {
    type Err = String;

    /// Parse a string into a Fixed4 value with up to 4 decimal places of precision.
    ///
    /// # Examples
    /// ```
    /// use transaction_processor::Fixed4;
    /// use std::str::FromStr;
    ///
    /// // Using FromStr trait
    /// let amount: Fixed4 = "123.45".parse().unwrap();
    /// assert_eq!(amount.to_string(), "123.4500");
    ///
    /// // Using FromStr::from_str directly
    /// let amount = Fixed4::from_str("0.0001").unwrap();
    /// assert_eq!(amount.to_string(), "0.0001");
    ///
    /// // Negative amounts
    /// let amount: Fixed4 = "-50.25".parse().unwrap();
    /// assert_eq!(amount.to_string(), "-50.2500");
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        // Handle empty string
        if value.is_empty() {
            return Err("Empty string".to_string());
        }

        // Handle negative numbers
        let (is_negative, value) = if let Some(stripped) = value.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, value)
        };

        // Split on decimal point
        let parts: Vec<&str> = value.split('.').collect();

        let result = match parts.len() {
            1 => {
                // No decimal point, parse as whole number
                let whole: i64 = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid number: {}", value))?;
                Ok(Self(whole * Self::SCALE))
            }
            2 => {
                // Has decimal point
                let whole: i64 = if parts[0].is_empty() {
                    0
                } else {
                    parts[0]
                        .parse()
                        .map_err(|_| format!("Invalid whole number: {}", parts[0]))?
                };

                let decimal_str = parts[1];
                if decimal_str.len() > 4 {
                    return Err(format!(
                        "Too many decimal places: {} (max 4)",
                        decimal_str.len()
                    ));
                }

                // Pad with zeros to get exactly 4 decimal places
                let padded_decimal = format!("{:0<4}", decimal_str);
                let decimal: i64 = padded_decimal
                    .parse()
                    .map_err(|_| format!("Invalid decimal: {}", decimal_str))?;

                Ok(Self(whole * Self::SCALE + decimal))
            }
            _ => Err(format!(
                "Invalid format: {} (multiple decimal points)",
                value
            )),
        };

        // Apply negative sign if needed
        match result {
            Ok(Self(val)) => Ok(Self(if is_negative { -val } else { val })),
            Err(e) => Err(e),
        }
    }
}

impl Fixed4 {
    /// Convert to f64 for compatibility (may lose precision for very large values)
    /// 
    /// Note: For display purposes, prefer using `to_string()` or the `Display` trait
    /// to avoid any potential floating-point precision issues.
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }
}

impl std::fmt::Display for Fixed4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 0 {
            let abs_val = self.0.abs();
            let whole = abs_val / Self::SCALE;
            let decimal = abs_val % Self::SCALE;
            write!(f, "-{}.{:04}", whole, decimal)
        } else {
            let whole = self.0 / Self::SCALE;
            let decimal = self.0 % Self::SCALE;
            write!(f, "{}.{:04}", whole, decimal)
        }
    }
}

impl std::ops::Add for Fixed4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Fixed4(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Fixed4 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Fixed4 {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}
