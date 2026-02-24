use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComplianceReport {
    pub id: String,
    pub generated_at: DateTime<Utc>,
    pub report_type: String,
    pub data: String, // JSON or structured data
    pub filing_deadline: Option<DateTime<Utc>>,
    pub status: ReportStatus,
    pub certifications: Vec<Certification>,
    pub audit_trail: Vec<AuditEntry>,
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ReportStatus {
    Draft,
    Filed,
    Audited,
    Certified,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Certification {
    pub cert_type: String,
    pub issued_by: String,
    pub issued_at: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditEntry {
    pub auditor: String,
    pub comment: String,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

impl ComplianceReport {
    pub fn generate_signature(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(serialized);
        format!("{:x}", hasher.finalize())
    }
}


use std::collections::HashMap;

#[derive(Default)]
pub struct ComplianceSystem {
    pub reports: HashMap<String, ComplianceReport>,
    pub filing_deadlines: HashMap<String, DateTime<Utc>>,
}

impl ComplianceSystem {
    pub fn new() -> Self {
        Self::default()
    }

    // Generate and store a new report
    pub fn generate_report(&mut self, report: ComplianceReport) {
        self.reports.insert(report.id.clone(), report);
    }

    // File a report (update status and track deadline)
    pub fn file_report(&mut self, report_id: &str, deadline: DateTime<Utc>) -> Result<(), String> {
        if let Some(report) = self.reports.get_mut(report_id) {
            report.status = ReportStatus::Filed;
            report.filing_deadline = Some(deadline);
            self.filing_deadlines.insert(report_id.to_string(), deadline);
            Ok(())
        } else {
            Err("Report not found".to_string())
        }
    }

    // Add an audit entry
    pub fn add_audit_entry(&mut self, report_id: &str, entry: AuditEntry) -> Result<(), String> {
        if let Some(report) = self.reports.get_mut(report_id) {
            report.audit_trail.push(entry);
            report.status = ReportStatus::Audited;
            Ok(())
        } else {
            Err("Report not found".to_string())
        }
    }

    // Add a certification
    pub fn add_certification(&mut self, report_id: &str, cert: Certification) -> Result<(), String> {
        if let Some(report) = self.reports.get_mut(report_id) {
            report.certifications.push(cert);
            report.status = ReportStatus::Certified;
            Ok(())
        } else {
            Err("Report not found".to_string())
        }
    }

    // Get compliance analytics (e.g., counts by status)
    pub fn analytics(&self) -> HashMap<ReportStatus, usize> {
        let mut stats = HashMap::new();
        for report in self.reports.values() {
            *stats.entry(report.status.clone()).or_insert(0) += 1;
        }
        stats
    }

    // Track overdue filings
    pub fn overdue_reports(&self, now: DateTime<Utc>) -> Vec<String> {
        self.filing_deadlines.iter()
            .filter(|(_, &deadline)| deadline < now)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
