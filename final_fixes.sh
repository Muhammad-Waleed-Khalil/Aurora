#!/bin/bash
cd /home/user/Aurora/crates/aurora_parser/src

echo "Applying final fixes..."

# Fix 1: Remove tuple pattern matching from TokenKind::IntLiteral and FloatLiteral in exprs.rs
sed -i 's/TokenKind::IntLiteral(n) =>/TokenKind::IntLiteral =>/g' exprs.rs
sed -i 's/TokenKind::FloatLiteral(f) =>/TokenKind::FloatLiteral =>/g' exprs.rs

# Fix 2: Same for patterns.rs
sed -i 's/TokenKind::IntLiteral(n) =>/TokenKind::IntLiteral =>/g' patterns.rs
sed -i 's/TokenKind::FloatLiteral(f) =>/TokenKind::FloatLiteral =>/g' patterns.rs

# Fix 3: AndAmpersand -> AndAnd in exprs.rs
sed -i 's/TokenKind::AndAmpersand/TokenKind::AndAnd/g' exprs.rs

echo "✓ Token literal fixes applied"
