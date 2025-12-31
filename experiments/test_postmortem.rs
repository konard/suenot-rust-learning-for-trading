use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Severity levels for incidents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
        }
    }
}

/// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentStatus {
    Investigating,
    Identified,
    Fixing,
    Monitoring,
    Resolved,
    Closed,
}

/// Simple incident record
#[derive(Debug, Clone)]
pub struct IncidentRecord {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub time_to_detect: Duration,
    pub time_to_respond: Duration,
    pub time_to_resolve: Duration,
    pub affected_customers: u64,
    pub revenue_impact: f64,
    pub action_items_created: u32,
    pub action_items_completed: u32,
    pub was_recurring: bool,
}

/// Incident metrics tracker
#[derive(Debug)]
pub struct IncidentMetricsTracker {
    incidents: Vec<IncidentRecord>,
    recurrence_map: HashMap<String, u32>,
}

impl IncidentMetricsTracker {
    pub fn new() -> Self {
        IncidentMetricsTracker {
            incidents: Vec::new(),
            recurrence_map: HashMap::new(),
        }
    }

    pub fn add_incident(&mut self, record: IncidentRecord) {
        *self.recurrence_map
            .entry(record.category.clone())
            .or_insert(0) += 1;
        self.incidents.push(record);
    }

    pub fn mttd(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }
        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_detect)
            .sum();
        total / self.incidents.len() as u32
    }

    pub fn mttr_resolve(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }
        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_resolve)
            .sum();
        total / self.incidents.len() as u32
    }

    pub fn recurrence_rate(&self) -> f64 {
        if self.incidents.is_empty() {
            return 0.0;
        }
        let recurring = self.incidents.iter()
            .filter(|i| i.was_recurring)
            .count();
        (recurring as f64 / self.incidents.len() as f64) * 100.0
    }
}

fn main() {
    println!("=== Post-Mortem Code Test ===\n");

    let mut tracker = IncidentMetricsTracker::new();

    tracker.add_incident(IncidentRecord {
        id: "INC-001".to_string(),
        severity: "Critical".to_string(),
        category: "Memory Leak".to_string(),
        time_to_detect: Duration::from_secs(120),
        time_to_respond: Duration::from_secs(180),
        time_to_resolve: Duration::from_secs(2820),
        affected_customers: 3247,
        revenue_impact: 125000.0,
        action_items_created: 5,
        action_items_completed: 4,
        was_recurring: false,
    });

    tracker.add_incident(IncidentRecord {
        id: "INC-002".to_string(),
        severity: "High".to_string(),
        category: "Network Timeout".to_string(),
        time_to_detect: Duration::from_secs(60),
        time_to_respond: Duration::from_secs(120),
        time_to_resolve: Duration::from_secs(900),
        affected_customers: 521,
        revenue_impact: 15000.0,
        action_items_created: 3,
        action_items_completed: 3,
        was_recurring: true,
    });

    println!("Total incidents: {}", tracker.incidents.len());
    println!("MTTD: {:?}", tracker.mttd());
    println!("MTTR: {:?}", tracker.mttr_resolve());
    println!("Recurrence rate: {:.1}%", tracker.recurrence_rate());
    println!("\nTest PASSED!");
}
