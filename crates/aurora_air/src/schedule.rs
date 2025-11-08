//! Instruction Scheduling

use crate::air::{AirFunction, Instruction};
use std::collections::HashMap;

/// CPU Profile for instruction costs
#[derive(Debug, Clone)]
pub struct CpuProfile {
    pub name: String,
    pub latencies: HashMap<String, u32>,
}

impl CpuProfile {
    /// Skylake profile
    pub fn skylake() -> Self {
        let mut latencies = HashMap::new();
        latencies.insert("mov".to_string(), 1);
        latencies.insert("add".to_string(), 1);
        latencies.insert("mul".to_string(), 3);
        latencies.insert("div".to_string(), 20);
        latencies.insert("load".to_string(), 5);
        latencies.insert("store".to_string(), 1);
        
        Self {
            name: "Skylake".to_string(),
            latencies,
        }
    }

    /// Zen profile
    pub fn zen() -> Self {
        let mut latencies = HashMap::new();
        latencies.insert("mov".to_string(), 1);
        latencies.insert("add".to_string(), 1);
        latencies.insert("mul".to_string(), 4);
        latencies.insert("div".to_string(), 24);
        latencies.insert("load".to_string(), 4);
        latencies.insert("store".to_string(), 1);
        
        Self {
            name: "Zen".to_string(),
            latencies,
        }
    }

    fn latency(&self, inst: &Instruction) -> u32 {
        let key = match inst {
            Instruction::Mov { .. } => "mov",
            Instruction::Add { .. } => "add",
            Instruction::Imul { .. } => "mul",
            Instruction::Idiv { .. } => "div",
            _ => "mov", // Default
        };
        
        *self.latencies.get(key).unwrap_or(&1)
    }
}

/// Instruction scheduler
pub struct InstructionScheduler {
    profile: CpuProfile,
    scheduled_count: usize,
}

impl InstructionScheduler {
    pub fn new(profile: CpuProfile) -> Self {
        Self {
            profile,
            scheduled_count: 0,
        }
    }

    /// Schedule instructions in function
    pub fn schedule(&mut self, func: &mut AirFunction) {
        // Simplified scheduling: reorder independent instructions
        // In reality, would build dependency graph and schedule optimally
        
        let mut scheduled = Vec::new();
        let mut pending = func.instructions.clone();
        
        while !pending.is_empty() {
            // Find instruction with minimal dependencies
            if let Some(next_idx) = self.find_next_instruction(&pending, &scheduled) {
                let inst = pending.remove(next_idx);
                scheduled.push(inst);
                self.scheduled_count += 1;
            } else {
                // No more optimizations possible
                scheduled.extend(pending);
                break;
            }
        }
        
        func.instructions = scheduled;
    }

    fn find_next_instruction(
        &self,
        pending: &[Instruction],
        _scheduled: &[Instruction],
    ) -> Option<usize> {
        // Simplified: prefer low-latency instructions
        let mut best_idx = None;
        let mut best_latency = u32::MAX;
        
        for (i, inst) in pending.iter().enumerate() {
            let latency = self.profile.latency(inst);
            if latency < best_latency {
                best_latency = latency;
                best_idx = Some(i);
            }
        }
        
        best_idx
    }

    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{Operand, Register};

    #[test]
    fn test_cpu_profiles() {
        let skylake = CpuProfile::skylake();
        assert_eq!(skylake.name, "Skylake");
        
        let zen = CpuProfile::zen();
        assert_eq!(zen.name, "Zen");
    }

    #[test]
    fn test_scheduler() {
        let mut scheduler = InstructionScheduler::new(CpuProfile::skylake());
        let mut func = AirFunction::new("test".to_string());
        
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1),
        });
        
        scheduler.schedule(&mut func);
        assert!(scheduler.scheduled_count() > 0);
    }
}
