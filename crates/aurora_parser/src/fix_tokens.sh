#!/bin/bash
# Fix TokenKind variant names in all parser files

for file in decls.rs exprs.rs patterns.rs stmts.rs types.rs; do
    if [ -f "$file" ]; then
        # Backup
        cp "$file" "$file.bak"
        
        # Replace token names
        sed -i 's/TokenKind::Identifier/TokenKind::Ident/g' "$file"
        sed -i 's/TokenKind::Arrow/TokenKind::RArrow/g' "$file"
        sed -i 's/TokenKind::EqualEqual/TokenKind::EqEq/g' "$file"
        sed -i 's/TokenKind::BangEqual/TokenKind::NotEq/g' "$file"
        sed -i 's/TokenKind::Bang/TokenKind::Not/g' "$file"
        sed -i 's/TokenKind::Ampersand/TokenKind::And/g' "$file"
        sed -i 's/TokenKind::DoubleStar/TokenKind::StarStar/g' "$file"
        sed -i 's/TokenKind::AmpersandAmpersand/TokenKind::AndAnd/g' "$file"
        sed -i 's/TokenKind::PipePipe/TokenKind::OrOr/g' "$file"
        sed -i 's/TokenKind::Pipe/TokenKind::Or/g' "$file"
        sed -i 's/TokenKind::DoubleColon/TokenKind::ColonColon/g' "$file"
        sed -i 's/TokenKind::AmpersandEq/TokenKind::AndEq/g' "$file"
        sed -i 's/TokenKind::PipeEq/TokenKind::OrEq/g' "$file"
        sed -i 's/TokenKind::Where/TokenKind::Where/g' "$file"
        sed -i 's/TokenKind::In/TokenKind::In/g' "$file"
        
        echo "Fixed $file"
    fi
done
