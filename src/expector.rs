use regex::Regex;

#[derive(Debug, Clone)]
pub struct Expector {
    pub value: String,
    pub negative: bool,
}

impl Expector {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
            negative: false,
        }
    }

    pub fn not(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn to_be(&mut self, expected: &str) -> Result<(), String> {
        let condition = self.value == expected.to_string();

        if !condition && !self.negative {
            let error = format!(
                "Expected value to be {} but instead got {}",
                expected.to_string(),
                self.value.to_string()
            );

            Err(error)
        } else if condition && self.negative {
            let error = format!(
                "Expected value {} to not be {} but it was",
                self.value.to_string(),
                expected.to_string()
            );

            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn to_match(&mut self, pattern: &str) -> Result<(), String> {
        let regex = Regex::new(pattern).unwrap();
        let condition = regex.is_match(&self.value);

        if !condition && !self.negative {
            let error = format!(
                "Expected value {} to match pattern {} but it did not",
                self.value.to_string(),
                pattern.to_string()
            );

            Err(error)
        } else if condition && self.negative {
            let error = format!(
                "Expected value {} to not match pattern {} but it did",
                self.value.to_string(),
                pattern.to_string()
            );

            Err(error)
        } else {
            Ok(())
        }
    }
}
