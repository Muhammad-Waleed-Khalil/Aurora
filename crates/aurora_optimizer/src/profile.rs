//! CPU-specific optimization profiles

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CPU microarchitecture profile
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuProfile {
    /// Intel Skylake and newer
    Skylake,
    /// AMD Zen 2 and newer
    Zen2,
    /// AMD Zen 3 and newer
    Zen3,
    /// ARM Cortex-A76 and newer
    CortexA76,
    /// Apple Silicon (M1/M2/M3)
    AppleSilicon,
    /// Generic x86_64
    Generic,
}

/// Optimization characteristics for CPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCharacteristics {
    /// CPU profile name
    pub profile: CpuProfile,
    /// Cache line size in bytes
    pub cache_line_size: usize,
    /// L1 data cache size in KB
    pub l1_cache_kb: usize,
    /// L2 cache size in KB
    pub l2_cache_kb: usize,
    /// L3 cache size in KB (if present)
    pub l3_cache_kb: Option<usize>,
    /// Register count
    pub register_count: usize,
    /// SIMD width in bytes (e.g., 32 for AVX2)
    pub simd_width: usize,
    /// Branch predictor accuracy (0.0-1.0)
    pub branch_predictor_accuracy: f32,
}

impl CpuCharacteristics {
    /// Get characteristics for Skylake
    pub fn skylake() -> Self {
        Self {
            profile: CpuProfile::Skylake,
            cache_line_size: 64,
            l1_cache_kb: 32,
            l2_cache_kb: 256,
            l3_cache_kb: Some(8192),
            register_count: 16,
            simd_width: 32, // AVX2
            branch_predictor_accuracy: 0.95,
        }
    }

    /// Get characteristics for Zen3
    pub fn zen3() -> Self {
        Self {
            profile: CpuProfile::Zen3,
            cache_line_size: 64,
            l1_cache_kb: 32,
            l2_cache_kb: 512,
            l3_cache_kb: Some(32768),
            register_count: 16,
            simd_width: 32, // AVX2
            branch_predictor_accuracy: 0.96,
        }
    }

    /// Get characteristics for Apple Silicon
    pub fn apple_silicon() -> Self {
        Self {
            profile: CpuProfile::AppleSilicon,
            cache_line_size: 128,
            l1_cache_kb: 192,
            l2_cache_kb: 12288,
            l3_cache_kb: None,
            register_count: 32,
            simd_width: 16, // NEON 128-bit
            branch_predictor_accuracy: 0.97,
        }
    }

    /// Get generic characteristics
    pub fn generic() -> Self {
        Self {
            profile: CpuProfile::Generic,
            cache_line_size: 64,
            l1_cache_kb: 32,
            l2_cache_kb: 256,
            l3_cache_kb: Some(4096),
            register_count: 16,
            simd_width: 16, // SSE2
            branch_predictor_accuracy: 0.90,
        }
    }
}

/// Optimization strategy based on CPU profile
#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    /// Target CPU characteristics
    pub cpu: CpuCharacteristics,
    /// Prefer inlining
    pub prefer_inline: bool,
    /// Unroll loops
    pub loop_unrolling: bool,
    /// Vectorize loops
    pub vectorization: bool,
    /// Optimize for size
    pub optimize_size: bool,
    /// Custom tuning parameters
    pub tuning_params: HashMap<String, f32>,
}

impl OptimizationStrategy {
    /// Create strategy for CPU profile
    pub fn for_cpu(cpu: CpuCharacteristics) -> Self {
        let vectorization = cpu.simd_width >= 16;
        let prefer_inline = cpu.branch_predictor_accuracy > 0.94;

        Self {
            cpu,
            prefer_inline,
            loop_unrolling: true,
            vectorization,
            optimize_size: false,
            tuning_params: HashMap::new(),
        }
    }

    /// Set tuning parameter
    pub fn with_param(mut self, key: String, value: f32) -> Self {
        self.tuning_params.insert(key, value);
        self
    }

    /// Get inline threshold
    pub fn inline_threshold(&self) -> usize {
        if self.prefer_inline {
            1000
        } else {
            500
        }
    }

    /// Get loop unroll factor
    pub fn unroll_factor(&self) -> usize {
        if self.loop_unrolling {
            match self.cpu.profile {
                CpuProfile::AppleSilicon => 8,
                CpuProfile::Zen3 => 6,
                CpuProfile::Skylake => 4,
                _ => 4,
            }
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_characteristics() {
        let skylake = CpuCharacteristics::skylake();
        assert_eq!(skylake.profile, CpuProfile::Skylake);
        assert_eq!(skylake.cache_line_size, 64);
        assert_eq!(skylake.simd_width, 32);
    }

    #[test]
    fn test_zen3_characteristics() {
        let zen3 = CpuCharacteristics::zen3();
        assert_eq!(zen3.l2_cache_kb, 512);
        assert_eq!(zen3.branch_predictor_accuracy, 0.96);
    }

    #[test]
    fn test_apple_silicon_characteristics() {
        let apple = CpuCharacteristics::apple_silicon();
        assert_eq!(apple.cache_line_size, 128);
        assert_eq!(apple.register_count, 32);
    }

    #[test]
    fn test_optimization_strategy() {
        let cpu = CpuCharacteristics::skylake();
        let strategy = OptimizationStrategy::for_cpu(cpu);

        assert!(strategy.vectorization);
        assert!(strategy.loop_unrolling);
        assert_eq!(strategy.unroll_factor(), 4);
    }

    #[test]
    fn test_inline_threshold() {
        let cpu = CpuCharacteristics::skylake();
        let strategy = OptimizationStrategy::for_cpu(cpu);

        assert_eq!(strategy.inline_threshold(), 1000);
    }

    #[test]
    fn test_tuning_params() {
        let cpu = CpuCharacteristics::generic();
        let strategy = OptimizationStrategy::for_cpu(cpu)
            .with_param("custom_param".to_string(), 1.5);

        assert_eq!(strategy.tuning_params.get("custom_param"), Some(&1.5));
    }

    #[test]
    fn test_unroll_factors() {
        let apple = CpuCharacteristics::apple_silicon();
        let strategy = OptimizationStrategy::for_cpu(apple);
        assert_eq!(strategy.unroll_factor(), 8);

        let zen3 = CpuCharacteristics::zen3();
        let strategy = OptimizationStrategy::for_cpu(zen3);
        assert_eq!(strategy.unroll_factor(), 6);
    }
}
