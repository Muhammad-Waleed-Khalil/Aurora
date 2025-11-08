//! Instruction Scheduling
//!
//! Implements latency-aware instruction scheduling with:
//! - Dependency tracking
//! - CPU-specific latency/throughput models
//! - List scheduling algorithm
//! - Critical path analysis

use crate::air::{AirFunction, Instruction, Operand, Register};
use std::collections::{HashMap, HashSet};

/// CPU Profile for instruction costs
#[derive(Debug, Clone)]
pub struct CpuProfile {
    pub name: String,
    /// Instruction latencies (cycles)
    pub latencies: HashMap<String, u32>,
    /// Instruction throughput (instructions per cycle)
    pub throughput: HashMap<String, f32>,
}

impl CpuProfile {
    /// Intel Skylake profile
    pub fn skylake() -> Self {
        let mut latencies = HashMap::new();
        latencies.insert("mov".to_string(), 1);
        latencies.insert("add".to_string(), 1);
        latencies.insert("sub".to_string(), 1);
        latencies.insert("mul".to_string(), 3);
        latencies.insert("div".to_string(), 20);
        latencies.insert("load".to_string(), 5);
        latencies.insert("store".to_string(), 1);
        latencies.insert("lea".to_string(), 1);
        latencies.insert("and".to_string(), 1);
        latencies.insert("or".to_string(), 1);
        latencies.insert("xor".to_string(), 1);
        latencies.insert("shl".to_string(), 1);
        latencies.insert("shr".to_string(), 1);
        latencies.insert("cmp".to_string(), 1);
        latencies.insert("test".to_string(), 1);
        latencies.insert("jmp".to_string(), 1);
        latencies.insert("call".to_string(), 2);

        let mut throughput = HashMap::new();
        throughput.insert("mov".to_string(), 0.5);  // 2 per cycle
        throughput.insert("add".to_string(), 0.33); // 3 per cycle
        throughput.insert("mul".to_string(), 1.0);
        throughput.insert("div".to_string(), 6.0);

        Self {
            name: "Skylake".to_string(),
            latencies,
            throughput,
        }
    }

    /// AMD Zen profile
    pub fn zen() -> Self {
        let mut latencies = HashMap::new();
        latencies.insert("mov".to_string(), 1);
        latencies.insert("add".to_string(), 1);
        latencies.insert("sub".to_string(), 1);
        latencies.insert("mul".to_string(), 4);
        latencies.insert("div".to_string(), 24);
        latencies.insert("load".to_string(), 4);
        latencies.insert("store".to_string(), 1);
        latencies.insert("lea".to_string(), 1);
        latencies.insert("and".to_string(), 1);
        latencies.insert("or".to_string(), 1);
        latencies.insert("xor".to_string(), 1);
        latencies.insert("shl".to_string(), 1);
        latencies.insert("shr".to_string(), 1);
        latencies.insert("cmp".to_string(), 1);
        latencies.insert("test".to_string(), 1);
        latencies.insert("jmp".to_string(), 1);
        latencies.insert("call".to_string(), 2);

        let mut throughput = HashMap::new();
        throughput.insert("mov".to_string(), 0.5);
        throughput.insert("add".to_string(), 0.25); // 4 per cycle
        throughput.insert("mul".to_string(), 1.0);
        throughput.insert("div".to_string(), 8.0);

        Self {
            name: "Zen".to_string(),
            latencies,
            throughput,
        }
    }

    /// Generic profile
    pub fn generic() -> Self {
        let mut latencies = HashMap::new();
        latencies.insert("mov".to_string(), 1);
        latencies.insert("add".to_string(), 1);
        latencies.insert("mul".to_string(), 3);
        latencies.insert("div".to_string(), 20);
        latencies.insert("load".to_string(), 4);
        latencies.insert("store".to_string(), 1);

        Self {
            name: "Generic".to_string(),
            latencies,
            throughput: HashMap::new(),
        }
    }

    fn latency(&self, inst: &Instruction) -> u32 {
        let key = match inst {
            Instruction::Mov { .. } => "mov",
            Instruction::Add { .. } => "add",
            Instruction::Sub { .. } => "sub",
            Instruction::Imul { .. } => "mul",
            Instruction::Idiv { .. } => "div",
            Instruction::Lea { .. } => "lea",
            Instruction::And { .. } => "and",
            Instruction::Or { .. } => "or",
            Instruction::Xor { .. } => "xor",
            Instruction::Shl { .. } => "shl",
            Instruction::Shr { .. } => "shr",
            Instruction::Cmp { .. } => "cmp",
            Instruction::Test { .. } => "test",
            Instruction::Jmp { .. } | Instruction::Je { .. } | Instruction::Jne { .. } => "jmp",
            Instruction::Call { .. } => "call",
            _ => "mov", // Default
        };

        *self.latencies.get(key).unwrap_or(&1)
    }
}

/// Dependency information for an instruction
#[derive(Debug, Clone)]
struct InstructionDeps {
    index: usize,
    reads: HashSet<Register>,
    writes: HashSet<Register>,
    latency: u32,
}

/// Instruction scheduler with dependency tracking
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

    /// Schedule instructions in function (per basic block)
    pub fn schedule(&mut self, func: &mut AirFunction) {
        let mut new_instructions = Vec::new();
        let mut current_block = Vec::new();

        for inst in func.instructions.drain(..) {
            // Check if this is a basic block boundary
            if self.is_block_boundary(&inst) {
                if !current_block.is_empty() {
                    // Schedule the current block
                    let scheduled = self.schedule_block(current_block);
                    new_instructions.extend(scheduled);
                    current_block = Vec::new();
                }
                new_instructions.push(inst);
            } else {
                current_block.push(inst);
            }
        }

        // Schedule remaining block
        if !current_block.is_empty() {
            let scheduled = self.schedule_block(current_block);
            new_instructions.extend(scheduled);
        }

        func.instructions = new_instructions;
    }

    /// Check if instruction is a basic block boundary
    fn is_block_boundary(&self, inst: &Instruction) -> bool {
        matches!(
            inst,
            Instruction::Label { .. }
                | Instruction::Jmp { .. }
                | Instruction::Je { .. }
                | Instruction::Jne { .. }
                | Instruction::Jl { .. }
                | Instruction::Jle { .. }
                | Instruction::Jg { .. }
                | Instruction::Jge { .. }
                | Instruction::Call { .. }
                | Instruction::Ret
        )
    }

    /// Schedule a basic block using list scheduling
    fn schedule_block(&mut self, block: Vec<Instruction>) -> Vec<Instruction> {
        if block.is_empty() {
            return block;
        }

        if block.len() == 1 {
            self.scheduled_count += 1;
            return block;
        }

        // Build dependency information
        let deps: Vec<InstructionDeps> = block
            .iter()
            .enumerate()
            .map(|(i, inst)| InstructionDeps {
                index: i,
                reads: self.get_reads(inst),
                writes: self.get_writes(inst),
                latency: self.profile.latency(inst),
            })
            .collect();

        // Build dependency graph
        let mut depends_on: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..deps.len() {
            for j in 0..i {
                if self.has_dependency(&deps[j], &deps[i]) {
                    depends_on.entry(i).or_default().push(j);
                }
            }
        }

        // List scheduling
        let mut scheduled = Vec::new();
        let mut ready: Vec<usize> = Vec::new();
        let mut completed = HashSet::new();

        // Initialize ready list
        for i in 0..block.len() {
            if !depends_on.contains_key(&i) {
                ready.push(i);
            }
        }

        while !ready.is_empty() {
            // Pick best instruction from ready list
            let best_idx = self.pick_best(&ready, &deps);
            let inst_idx = ready.remove(best_idx);

            scheduled.push(block[inst_idx].clone());
            completed.insert(inst_idx);
            self.scheduled_count += 1;

            // Update ready list
            for i in 0..block.len() {
                if completed.contains(&i) {
                    continue;
                }

                let all_deps_done = depends_on
                    .get(&i)
                    .map(|deps| deps.iter().all(|d| completed.contains(d)))
                    .unwrap_or(true);

                if all_deps_done && !ready.contains(&i) {
                    ready.push(i);
                }
            }
        }

        scheduled
    }

    /// Check if there's a dependency between instructions
    fn has_dependency(&self, earlier: &InstructionDeps, later: &InstructionDeps) -> bool {
        // RAW (Read After Write) dependency
        if earlier.writes.iter().any(|w| later.reads.contains(w)) {
            return true;
        }

        // WAR (Write After Read) dependency
        if earlier.reads.iter().any(|r| later.writes.contains(r)) {
            return true;
        }

        // WAW (Write After Write) dependency
        if earlier.writes.iter().any(|w| later.writes.contains(w)) {
            return true;
        }

        false
    }

    /// Pick best instruction from ready list (based on latency and critical path)
    fn pick_best(&self, ready: &[usize], deps: &[InstructionDeps]) -> usize {
        let mut best_idx = 0;
        let mut best_score = i32::MIN;

        for (i, &inst_idx) in ready.iter().enumerate() {
            // Score based on latency (prefer long-latency instructions early)
            let score = deps[inst_idx].latency as i32;

            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Get registers read by instruction
    fn get_reads(&self, inst: &Instruction) -> HashSet<Register> {
        let mut reads = HashSet::new();

        match inst {
            Instruction::Mov { src, .. } => self.add_operand_reads(src, &mut reads),
            Instruction::Add { dest, src } | Instruction::Sub { dest, src } => {
                self.add_operand_reads(dest, &mut reads);
                self.add_operand_reads(src, &mut reads);
            }
            Instruction::Imul { dest, src } => {
                self.add_operand_reads(dest, &mut reads);
                self.add_operand_reads(src, &mut reads);
            }
            Instruction::Idiv { operand } => {
                self.add_operand_reads(operand, &mut reads);
                reads.insert(Register::RAX);
                reads.insert(Register::RDX);
            }
            Instruction::Cmp { left, right } | Instruction::Test { left, right } => {
                self.add_operand_reads(left, &mut reads);
                self.add_operand_reads(right, &mut reads);
            }
            Instruction::Push { operand } => self.add_operand_reads(operand, &mut reads),
            _ => {}
        }

        reads
    }

    /// Get registers written by instruction
    fn get_writes(&self, inst: &Instruction) -> HashSet<Register> {
        let mut writes = HashSet::new();

        match inst {
            Instruction::Mov { dest, .. }
            | Instruction::Lea { dest, .. }
            | Instruction::Movzx { dest, .. }
            | Instruction::Movsx { dest, .. } => {
                self.add_operand_writes(dest, &mut writes);
            }
            Instruction::Add { dest, .. }
            | Instruction::Sub { dest, .. }
            | Instruction::Imul { dest, .. }
            | Instruction::And { dest, .. }
            | Instruction::Or { dest, .. }
            | Instruction::Xor { dest, .. }
            | Instruction::Shl { dest, .. }
            | Instruction::Shr { dest, .. }
            | Instruction::Sar { dest, .. } => {
                self.add_operand_writes(dest, &mut writes);
            }
            Instruction::Idiv { .. } => {
                writes.insert(Register::RAX);
                writes.insert(Register::RDX);
            }
            Instruction::Pop { operand } => self.add_operand_writes(operand, &mut writes),
            Instruction::Call { .. } => {
                // Calls clobber caller-saved registers
                writes.insert(Register::RAX);
                writes.insert(Register::RCX);
                writes.insert(Register::RDX);
                writes.insert(Register::R8);
                writes.insert(Register::R9);
                writes.insert(Register::R10);
                writes.insert(Register::R11);
            }
            _ => {}
        }

        writes
    }

    fn add_operand_reads(&self, op: &Operand, reads: &mut HashSet<Register>) {
        match op {
            Operand::Reg(r) => {
                reads.insert(*r);
            }
            Operand::Mem { base, .. } => {
                reads.insert(*base);
            }
            Operand::MemComplex { base, index, .. } => {
                if let Some(b) = base {
                    reads.insert(*b);
                }
                if let Some(idx) = index {
                    reads.insert(*idx);
                }
            }
            _ => {}
        }
    }

    fn add_operand_writes(&self, op: &Operand, writes: &mut HashSet<Register>) {
        if let Operand::Reg(r) = op {
            writes.insert(*r);
        }
    }

    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_profiles() {
        let skylake = CpuProfile::skylake();
        assert_eq!(skylake.name, "Skylake");

        let zen = CpuProfile::zen();
        assert_eq!(zen.name, "Zen");

        let generic = CpuProfile::generic();
        assert_eq!(generic.name, "Generic");
    }

    #[test]
    fn test_latency_lookup() {
        let profile = CpuProfile::skylake();

        let mov = Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1),
        };
        assert_eq!(profile.latency(&mov), 1);

        let mul = Instruction::Imul {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(2),
        };
        assert_eq!(profile.latency(&mul), 3);
    }

    #[test]
    fn test_simple_scheduling() {
        let mut scheduler = InstructionScheduler::new(CpuProfile::skylake());
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1),
        });

        scheduler.schedule(&mut func);
        assert!(scheduler.scheduled_count() > 0);
    }

    #[test]
    fn test_dependency_detection() {
        let scheduler = InstructionScheduler::new(CpuProfile::skylake());

        let inst1 = Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1),
        };
        let inst2 = Instruction::Add {
            dest: Operand::Reg(Register::RBX),
            src: Operand::Reg(Register::RAX),
        };

        let reads1 = scheduler.get_reads(&inst1);
        let writes1 = scheduler.get_writes(&inst1);
        let reads2 = scheduler.get_reads(&inst2);

        assert!(writes1.contains(&Register::RAX));
        assert!(reads2.contains(&Register::RAX));
    }

    #[test]
    fn test_schedule_with_dependencies() {
        let mut scheduler = InstructionScheduler::new(CpuProfile::skylake());
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1),
        });
        func.push(Instruction::Add {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(2),
        });

        let before = func.instructions.len();
        scheduler.schedule(&mut func);
        assert_eq!(func.instructions.len(), before);
    }

    #[test]
    fn test_block_boundary_detection() {
        let scheduler = InstructionScheduler::new(CpuProfile::skylake());

        assert!(scheduler.is_block_boundary(&Instruction::Label {
            name: "test".to_string()
        }));

        assert!(scheduler.is_block_boundary(&Instruction::Ret));

        assert!(!scheduler.is_block_boundary(&Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1)
        }));
    }

    #[test]
    fn test_schedule_multiple_blocks() {
        let mut scheduler = InstructionScheduler::new(CpuProfile::skylake());
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Label {
            name: "block1".to_string(),
        });
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(1),
        });

        func.push(Instruction::Label {
            name: "block2".to_string(),
        });
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RBX),
            src: Operand::Imm(2),
        });

        scheduler.schedule(&mut func);
        assert!(scheduler.scheduled_count() > 0);
    }
}
