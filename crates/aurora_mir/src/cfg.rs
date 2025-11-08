//! Control Flow Graph (CFG) for MIR
//!
//! The CFG represents the control flow structure of a function,
//! tracking predecessor and successor relationships between basic blocks.

use crate::mir::{BasicBlock, BlockId, Function, Instruction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Control Flow Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFG {
    /// Entry block
    pub entry: BlockId,
    /// Exit blocks (blocks with return)
    pub exits: Vec<BlockId>,
    /// Predecessor map: block -> predecessors
    pub predecessors: HashMap<BlockId, Vec<BlockId>>,
    /// Successor map: block -> successors
    pub successors: HashMap<BlockId, Vec<BlockId>>,
    /// Post-order traversal
    pub post_order: Vec<BlockId>,
    /// Reverse post-order (RPO) traversal
    pub reverse_post_order: Vec<BlockId>,
}

impl CFG {
    /// Build CFG from function
    pub fn build(func: &Function) -> Self {
        let mut cfg = Self {
            entry: func.entry,
            exits: Vec::new(),
            predecessors: HashMap::new(),
            successors: HashMap::new(),
            post_order: Vec::new(),
            reverse_post_order: Vec::new(),
        };

        // Build edges from instructions
        for (block_id, block) in &func.blocks {
            if let Some(term) = block.terminator() {
                match term {
                    Instruction::Branch {
                        then_block,
                        else_block,
                        ..
                    } => {
                        cfg.add_edge(*block_id, *then_block);
                        cfg.add_edge(*block_id, *else_block);
                    }
                    Instruction::Jump { target, .. } => {
                        cfg.add_edge(*block_id, *target);
                    }
                    Instruction::Return { .. } => {
                        cfg.exits.push(*block_id);
                    }
                    _ => {}
                }
            }
        }

        // Compute traversal orders
        cfg.compute_post_order(&func);
        cfg.reverse_post_order = cfg.post_order.iter().copied().rev().collect();

        cfg
    }

    /// Add edge from -> to
    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.successors.entry(from).or_default().push(to);
        self.predecessors.entry(to).or_default().push(from);
    }

    /// Compute post-order traversal using DFS
    fn compute_post_order(&mut self, func: &Function) {
        let mut visited = HashSet::new();
        let mut post_order = Vec::new();

        fn dfs(
            block: BlockId,
            successors: &HashMap<BlockId, Vec<BlockId>>,
            visited: &mut HashSet<BlockId>,
            post_order: &mut Vec<BlockId>,
        ) {
            if visited.contains(&block) {
                return;
            }
            visited.insert(block);

            if let Some(succs) = successors.get(&block) {
                for &succ in succs {
                    dfs(succ, successors, visited, post_order);
                }
            }

            post_order.push(block);
        }

        dfs(func.entry, &self.successors, &mut visited, &mut post_order);
        self.post_order = post_order;
    }

    /// Get predecessors of a block
    pub fn preds(&self, block: BlockId) -> &[BlockId] {
        self.predecessors.get(&block).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get successors of a block
    pub fn succs(&self, block: BlockId) -> &[BlockId] {
        self.successors.get(&block).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if block1 dominates block2
    pub fn dominates(&self, block1: BlockId, block2: BlockId, dom_tree: &DominatorTree) -> bool {
        dom_tree.dominates(block1, block2)
    }

    /// Get reachable blocks from entry
    pub fn reachable_blocks(&self) -> HashSet<BlockId> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.entry);

        while let Some(block) = queue.pop_front() {
            if reachable.insert(block) {
                for &succ in self.succs(block) {
                    queue.push_back(succ);
                }
            }
        }

        reachable
    }

    /// Find loops in CFG (natural loops)
    pub fn find_loops(&self) -> Vec<Loop> {
        let mut loops = Vec::new();
        let reachable = self.reachable_blocks();

        // Find back edges (edges to blocks that dominate the source)
        for &block in &reachable {
            for &succ in self.succs(block) {
                // Check if succ dominates block (back edge)
                if self.reverse_post_order.iter().position(|&b| b == succ)
                    < self.reverse_post_order.iter().position(|&b| b == block)
                {
                    // Found a back edge: block -> succ
                    let loop_blocks = self.natural_loop(block, succ);
                    loops.push(Loop {
                        header: succ,
                        latch: block,
                        blocks: loop_blocks,
                    });
                }
            }
        }

        loops
    }

    /// Compute natural loop for back edge (latch -> header)
    fn natural_loop(&self, latch: BlockId, header: BlockId) -> Vec<BlockId> {
        let mut loop_blocks = vec![header];
        let mut worklist = vec![latch];
        let mut in_loop = HashSet::new();
        in_loop.insert(header);

        while let Some(block) = worklist.pop() {
            if in_loop.insert(block) {
                loop_blocks.push(block);
                for &pred in self.preds(block) {
                    if !in_loop.contains(&pred) {
                        worklist.push(pred);
                    }
                }
            }
        }

        loop_blocks
    }
}

/// Loop in CFG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loop {
    /// Loop header block
    pub header: BlockId,
    /// Loop latch block (back edge source)
    pub latch: BlockId,
    /// All blocks in loop
    pub blocks: Vec<BlockId>,
}

/// Dominator Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominatorTree {
    /// Immediate dominator map: block -> idom
    pub idom: HashMap<BlockId, BlockId>,
    /// Dominance frontiers
    pub frontiers: HashMap<BlockId, Vec<BlockId>>,
}

impl DominatorTree {
    /// Compute dominator tree using Lengauer-Tarjan algorithm
    pub fn compute(cfg: &CFG) -> Self {
        let mut idom = HashMap::new();
        
        // Simplified algorithm: iterative dataflow
        // Initialize: entry dominates itself
        let blocks: Vec<BlockId> = cfg.reverse_post_order.clone();
        let entry = cfg.entry;
        
        idom.insert(entry, entry);
        
        // Iterate until convergence
        let mut changed = true;
        while changed {
            changed = false;
            
            for &block in &blocks {
                if block == entry {
                    continue;
                }
                
                let preds = cfg.preds(block);
                if preds.is_empty() {
                    continue;
                }
                
                // Find first processed predecessor
                let mut new_idom = None;
                for &pred in preds {
                    if idom.contains_key(&pred) {
                        new_idom = Some(pred);
                        break;
                    }
                }
                
                if let Some(mut new_idom_val) = new_idom {
                    // Intersect with other predecessors
                    for &pred in preds {
                        if idom.contains_key(&pred) && pred != new_idom_val {
                            new_idom_val = Self::intersect(&idom, &blocks, new_idom_val, pred);
                        }
                    }
                    
                    // Update if changed
                    if idom.get(&block) != Some(&new_idom_val) {
                        idom.insert(block, new_idom_val);
                        changed = true;
                    }
                }
            }
        }
        
        // Compute dominance frontiers
        let frontiers = Self::compute_frontiers(cfg, &idom);
        
        Self { idom, frontiers }
    }
    
    /// Intersect two nodes in dominator tree
    fn intersect(
        idom: &HashMap<BlockId, BlockId>,
        rpo: &[BlockId],
        mut b1: BlockId,
        mut b2: BlockId,
    ) -> BlockId {
        let pos = |b: BlockId| rpo.iter().position(|&x| x == b).unwrap();
        
        while b1 != b2 {
            while pos(b1) > pos(b2) {
                b1 = *idom.get(&b1).unwrap();
            }
            while pos(b2) > pos(b1) {
                b2 = *idom.get(&b2).unwrap();
            }
        }
        b1
    }
    
    /// Compute dominance frontiers
    fn compute_frontiers(
        cfg: &CFG,
        idom: &HashMap<BlockId, BlockId>,
    ) -> HashMap<BlockId, Vec<BlockId>> {
        let mut frontiers: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        
        for &block in cfg.reverse_post_order.iter() {
            let preds = cfg.preds(block);
            if preds.len() >= 2 {
                for &pred in preds {
                    let mut runner = pred;
                    while runner != *idom.get(&block).unwrap_or(&cfg.entry) {
                        frontiers.entry(runner).or_default().push(block);
                        runner = *idom.get(&runner).unwrap_or(&cfg.entry);
                    }
                }
            }
        }
        
        frontiers
    }

    /// Check if block1 dominates block2
    pub fn dominates(&self, block1: BlockId, block2: BlockId) -> bool {
        if block1 == block2 {
            return true;
        }

        let mut current = block2;
        while let Some(&idom) = self.idom.get(&current) {
            if idom == current {
                break; // Reached entry
            }
            if idom == block1 {
                return true;
            }
            current = idom;
        }

        false
    }

    /// Get immediate dominator of a block
    pub fn idom(&self, block: BlockId) -> Option<BlockId> {
        self.idom.get(&block).copied()
    }

    /// Get dominance frontier of a block
    pub fn frontier(&self, block: BlockId) -> &[BlockId] {
        self.frontiers.get(&block).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Function, Instruction, Operand, Span};
    use aurora_types::{EffectSet, Type};

    fn create_test_function() -> Function {
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        func.entry = 0;

        // Block 0: entry
        let mut block0 = BasicBlock::new(0);
        block0.push(Instruction::Jump {
            target: 1,
            span: Span::dummy(),
        });
        func.add_block(block0);

        // Block 1: branch
        let mut block1 = BasicBlock::new(1);
        block1.push(Instruction::Branch {
            cond: Operand::Const(crate::mir::Constant::Bool(true)),
            then_block: 2,
            else_block: 3,
            span: Span::dummy(),
        });
        func.add_block(block1);

        // Block 2: then
        let mut block2 = BasicBlock::new(2);
        block2.push(Instruction::Jump {
            target: 4,
            span: Span::dummy(),
        });
        func.add_block(block2);

        // Block 3: else
        let mut block3 = BasicBlock::new(3);
        block3.push(Instruction::Jump {
            target: 4,
            span: Span::dummy(),
        });
        func.add_block(block3);

        // Block 4: merge
        let mut block4 = BasicBlock::new(4);
        block4.push(Instruction::Return {
            value: None,
            span: Span::dummy(),
        });
        func.add_block(block4);

        func
    }

    #[test]
    fn test_cfg_build() {
        let func = create_test_function();
        let cfg = CFG::build(&func);

        assert_eq!(cfg.entry, 0);
        assert_eq!(cfg.exits, vec![4]);
        assert_eq!(cfg.succs(0), &[1]);
        assert_eq!(cfg.succs(1).len(), 2);
    }

    #[test]
    fn test_cfg_predecessors() {
        let func = create_test_function();
        let cfg = CFG::build(&func);

        assert_eq!(cfg.preds(1), &[0]);
        assert_eq!(cfg.preds(4).len(), 2); // From blocks 2 and 3
    }

    #[test]
    fn test_cfg_reachable() {
        let func = create_test_function();
        let cfg = CFG::build(&func);

        let reachable = cfg.reachable_blocks();
        assert_eq!(reachable.len(), 5); // All 5 blocks reachable
    }

    #[test]
    fn test_dominator_tree() {
        let func = create_test_function();
        let cfg = CFG::build(&func);
        let dom = DominatorTree::compute(&cfg);

        // Entry dominates itself
        assert!(dom.dominates(0, 0));
        // Entry dominates all blocks
        assert!(dom.dominates(0, 1));
        assert!(dom.dominates(0, 4));
    }

    #[test]
    fn test_post_order() {
        let func = create_test_function();
        let cfg = CFG::build(&func);

        assert!(!cfg.post_order.is_empty());
        assert!(!cfg.reverse_post_order.is_empty());
        assert_eq!(cfg.post_order.len(), cfg.reverse_post_order.len());
    }
}
