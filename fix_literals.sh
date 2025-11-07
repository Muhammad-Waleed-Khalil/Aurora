#!/bin/bash
cd /home/user/Aurora/crates/aurora_parser/src

# Fix exprs.rs - IntLiteral parsing
sed -i '/TokenKind::IntLiteral => {/{N;s/TokenKind::IntLiteral => {\n                let n = \*n;/TokenKind::IntLiteral => {\n                let n = self.current().lexeme.parse::<i64>().unwrap_or(0);/}' exprs.rs

# Fix exprs.rs - FloatLiteral parsing  
sed -i '/TokenKind::FloatLiteral => {/{N;s/TokenKind::FloatLiteral => {\n                let f = \*f;/TokenKind::FloatLiteral => {\n                let f = self.current().lexeme.parse::<f64>().unwrap_or(0.0);/}' exprs.rs

# Fix patterns.rs - IntLiteral parsing
sed -i '/TokenKind::IntLiteral => {/{N;s/TokenKind::IntLiteral => {\n                let n = \*n;/TokenKind::IntLiteral => {\n                let n = self.current().lexeme.parse::<i64>().unwrap_or(0);/}' patterns.rs

# Fix patterns.rs - FloatLiteral parsing
sed -i '/TokenKind::FloatLiteral => {/{N;s/TokenKind::FloatLiteral => {\n                let f = \*f;/TokenKind::FloatLiteral => {\n                let f = self.current().lexeme.parse::<f64>().unwrap_or(0.0);/}' patterns.rs

echo "Literal parsing fixed"
