//! Versioned rule model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule {
    pub id: String,
    pub base_id: String,
    pub version: u32,
    pub claim: String,
    pub applies_to: Vec<String>,
    pub priority: u32,
}

impl Rule {
    pub fn next_version(&self, new_claim: String) -> Self {
        let version = self.version + 1;
        Self {
            id: format!("{}:v{}", self.base_id, version),
            base_id: self.base_id.clone(),
            version,
            claim: new_claim,
            applies_to: self.applies_to.clone(),
            priority: self.priority,
        }
    }
}
