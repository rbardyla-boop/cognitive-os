//! Packet permission checks.

use super::epistemic_license::EpistemicLicense;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketPermissions {
    pub allowed_use: Vec<String>,
    pub forbidden_use: Vec<String>,
}

impl PacketPermissions {
    pub fn permits(&self, license: EpistemicLicense, requested_use: &str) -> bool {
        if self.forbidden_use.iter().any(|item| item == requested_use) {
            return false;
        }

        if !self.allowed_use.iter().any(|item| item == requested_use) {
            return false;
        }

        license.permits_use(requested_use)
    }
}
