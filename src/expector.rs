
#[derive(Debug, Clone)]
pub struct Expector {
    pub value: String
}

impl Expector {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string()
        }
    }

    pub fn to_be(&mut self, expected: &str)-> Result<(), String>{
        if self.value != expected.to_string() {
            let error = format!("Expected value to be {} but instead got {}", expected.to_string(), self.value.to_string());
            
            Err(error)
        } else {
            Ok(())
        }
    }
}