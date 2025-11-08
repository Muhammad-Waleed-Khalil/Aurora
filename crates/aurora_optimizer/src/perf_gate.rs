//! Performance gates and benchmark enforcement

use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Performance gate error
#[derive(Debug, Error)]
pub enum PerfError {
    /// Regression detected
    #[error("Performance regression: {metric} is {actual:.2}% worse than baseline {baseline:.2}%")]
    Regression {
        /// Metric name
        metric: String,
        /// Baseline value
        baseline: f64,
        /// Actual value
        actual: f64,
    },

    /// Benchmark failed
    #[error("Benchmark failed: {0}")]
    BenchmarkFailed(String),
}

/// Performance metric
#[derive(Debug, Clone)]
pub struct PerfMetric {
    /// Metric name
    pub name: String,
    /// Measured value
    pub value: f64,
    /// Unit (e.g., "ms", "MB", "ops/s")
    pub unit: String,
}

impl PerfMetric {
    /// Create new metric
    pub fn new(name: impl Into<String>, value: f64, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value,
            unit: unit.into(),
        }
    }

    /// Create throughput metric (ops per second)
    pub fn throughput(name: impl Into<String>, ops_per_sec: f64) -> Self {
        Self::new(name, ops_per_sec, "ops/s")
    }

    /// Create latency metric (milliseconds)
    pub fn latency(name: impl Into<String>, duration: Duration) -> Self {
        Self::new(name, duration.as_secs_f64() * 1000.0, "ms")
    }

    /// Create memory metric (megabytes)
    pub fn memory(name: impl Into<String>, bytes: usize) -> Self {
        Self::new(name, bytes as f64 / 1_048_576.0, "MB")
    }
}

/// Performance gate configuration
#[derive(Debug, Clone)]
pub struct PerfGate {
    /// Baseline metrics
    baseline: HashMap<String, f64>,
    /// Maximum allowed regression percentage
    max_regression_pct: f64,
}

impl PerfGate {
    /// Create new performance gate
    pub fn new(max_regression_pct: f64) -> Self {
        Self {
            baseline: HashMap::new(),
            max_regression_pct,
        }
    }

    /// Set baseline for metric
    pub fn set_baseline(&mut self, name: String, value: f64) {
        self.baseline.insert(name, value);
    }

    /// Check metric against baseline
    pub fn check(&self, metric: &PerfMetric) -> Result<(), PerfError> {
        if let Some(&baseline) = self.baseline.get(&metric.name) {
            let regression_pct = ((metric.value - baseline) / baseline) * 100.0;

            if regression_pct > self.max_regression_pct {
                return Err(PerfError::Regression {
                    metric: metric.name.clone(),
                    baseline,
                    actual: metric.value,
                });
            }
        }

        Ok(())
    }

    /// Check multiple metrics
    pub fn check_all(&self, metrics: &[PerfMetric]) -> Result<(), Vec<PerfError>> {
        let errors: Vec<PerfError> = metrics
            .iter()
            .filter_map(|m| self.check(m).err())
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchResult {
    /// Benchmark name
    pub name: String,
    /// Metrics collected
    pub metrics: Vec<PerfMetric>,
    /// Passed performance gate
    pub passed: bool,
}

impl BenchResult {
    /// Create new result
    pub fn new(name: String) -> Self {
        Self {
            name,
            metrics: Vec::new(),
            passed: true,
        }
    }

    /// Add metric
    pub fn add_metric(&mut self, metric: PerfMetric) {
        self.metrics.push(metric);
    }

    /// Mark as failed
    pub fn fail(&mut self) {
        self.passed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_metric_creation() {
        let metric = PerfMetric::new("test", 100.0, "ms");
        assert_eq!(metric.name, "test");
        assert_eq!(metric.value, 100.0);
        assert_eq!(metric.unit, "ms");
    }

    #[test]
    fn test_throughput_metric() {
        let metric = PerfMetric::throughput("compute", 1000.0);
        assert_eq!(metric.unit, "ops/s");
        assert_eq!(metric.value, 1000.0);
    }

    #[test]
    fn test_latency_metric() {
        let duration = Duration::from_millis(50);
        let metric = PerfMetric::latency("request", duration);
        assert_eq!(metric.unit, "ms");
        assert_eq!(metric.value, 50.0);
    }

    #[test]
    fn test_memory_metric() {
        let metric = PerfMetric::memory("heap", 2_097_152); // 2 MB
        assert_eq!(metric.unit, "MB");
        assert_eq!(metric.value, 2.0);
    }

    #[test]
    fn test_perf_gate_no_regression() {
        let mut gate = PerfGate::new(5.0); // 5% max regression
        gate.set_baseline("test".to_string(), 100.0);

        let metric = PerfMetric::new("test", 102.0, "ms");
        assert!(gate.check(&metric).is_ok());
    }

    #[test]
    fn test_perf_gate_regression() {
        let mut gate = PerfGate::new(5.0);
        gate.set_baseline("test".to_string(), 100.0);

        let metric = PerfMetric::new("test", 110.0, "ms"); // 10% worse
        assert!(gate.check(&metric).is_err());
    }

    #[test]
    fn test_perf_gate_no_baseline() {
        let gate = PerfGate::new(5.0);
        let metric = PerfMetric::new("unknown", 100.0, "ms");
        assert!(gate.check(&metric).is_ok()); // No baseline, no error
    }

    #[test]
    fn test_perf_gate_check_all() {
        let mut gate = PerfGate::new(5.0);
        gate.set_baseline("m1".to_string(), 100.0);
        gate.set_baseline("m2".to_string(), 50.0);

        let metrics = vec![
            PerfMetric::new("m1", 102.0, "ms"), // OK
            PerfMetric::new("m2", 60.0, "ms"),  // 20% regression
        ];

        let result = gate.check_all(&metrics);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn test_bench_result() {
        let mut result = BenchResult::new("test_bench".to_string());
        result.add_metric(PerfMetric::new("latency", 10.0, "ms"));

        assert_eq!(result.name, "test_bench");
        assert_eq!(result.metrics.len(), 1);
        assert!(result.passed);

        result.fail();
        assert!(!result.passed);
    }
}
