pub struct ParameterRange {
    default: f64,
    minimum: f64,
    maximum: f64,
}

impl ParameterRange {
    pub fn new(default: f64, minimum: f64, maximum: f64) -> Self {
        let range = Self {
            default,
            minimum,
            maximum,
        };

        assert!(range.is_valid());

        range
    }

    pub fn default(&self) -> f64 {
        self.default
    }

    pub fn is_valid(&self) -> bool {
        if self.maximum < self.minimum {
            return false;
        }

        if !(self.minimum..=self.maximum).contains(&self.default) {
            return false;
        }

        true
    }

    pub fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.minimum, self.maximum)
    }
}
