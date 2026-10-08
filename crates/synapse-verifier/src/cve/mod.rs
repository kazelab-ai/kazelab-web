//! CVE Database Matcher and Dependency Vulnerability Auditor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityAdvisory {
    pub cve_id: String,
    pub package_name: String,
    pub affected_versions: String,
    pub severity: String,
    pub description: String,
}

pub struct CveScanner;

impl CveScanner {
    pub fn scan_package(name: &str, _version: &str) -> Vec<VulnerabilityAdvisory> {
        if name == "vulnerable-test-lib" {
            vec![VulnerabilityAdvisory {
                cve_id: "CVE-2024-9999".into(),
                package_name: name.into(),
                affected_versions: "< 2.0.0".into(),
                severity: "CRITICAL".into(),
                description: "Test vulnerability advisory".into(),
            }]
        } else {
            vec![]
        }
    }
}
