use crate::ast::{BinOp, Expr, Stmt, UnOp};
use crate::error::{CalcError, Result};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, i: 0 }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.i].kind
    }

    fn peek_pos(&self) -> usize {
        self.tokens[self.i].pos
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.i].clone();
        self.i += 1;
        tok
    }

    fn eat(&mut self, want: &TokenKind) -> Result<Token> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(want) {
            Ok(self.advance())
        } else {
            Err(CalcError::SyntaxError {
                msg: format!("Ожидалось {:?}, но встретилось {:?}", want, self.peek()),
                pos: self.peek_pos(),
            })
        }
    }

    fn eat_ident(&mut self) -> Result<String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(s) => Ok(s),
            other => Err(CalcError::SyntaxError {
                msg: format!("Ожидался идентификатор, но встретилось {:?}", other),
                pos: tok.pos,
            }),
        }
    }

    pub fn parse_single_expr(&mut self) -> Result<Expr> {
        let e = self.parse_expr(0)?;
        self.eat(&TokenKind::Eof)?;
        Ok(e)
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            while matches!(self.peek(), TokenKind::Semicolon | TokenKind::Newline) {
                self.advance();
            }
            if matches!(self.peek(), TokenKind::Eof) {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        self.eat(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        loop {
            while matches!(self.peek(), TokenKind::Semicolon | TokenKind::Newline) {
                self.advance();
            }
            if matches!(self.peek(), TokenKind::RBrace) {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        self.eat(&TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek() {
            TokenKind::KwFn => {
                let pos = self.peek_pos();
                self.advance();
                let name = self.eat_ident()?;
                self.eat(&TokenKind::LParen)?;
                let mut params = Vec::new();
                if !matches!(self.peek(), TokenKind::RParen) {
                    loop {
                        params.push(self.eat_ident()?);
                        if matches!(self.peek(), TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.eat(&TokenKind::RParen)?;
                self.eat(&TokenKind::Eq)?;
                let body = self.parse_expr(0)?;
                Ok(Stmt::FnDef { name, params, body, pos })
            }
            TokenKind::KwAlias => {
                let pos = self.peek_pos();
                self.advance();
                let name = self.eat_ident()?;
                self.eat(&TokenKind::Eq)?;
                let target = self.eat_ident()?;
                Ok(Stmt::Alias { name, target, pos })
            }
            TokenKind::KwWhile => {
                let pos = self.peek_pos();
                self.advance();
                self.eat(&TokenKind::LParen)?;
                let cond = self.parse_expr(0)?;
                self.eat(&TokenKind::RParen)?;
                let body = self.parse_block()?;
                Ok(Stmt::While { cond, body, pos })
            }
            TokenKind::KwRepeat => {
                let pos = self.peek_pos();
                self.advance();
                let count = self.parse_expr(0)?;
                let body = self.parse_block()?;
                Ok(Stmt::Repeat { count, body, pos })
            }
            _ => Ok(Stmt::Expr(self.parse_expr(0)?)),
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = self.parse_nud()?;

        if min_bp == 0 && matches!(self.peek(), TokenKind::Eq) {
            if let Expr::Var(name, pos) = lhs {
                self.advance();
                let value = self.parse_expr(0)?;
                return Ok(Expr::Assign { name, value: Box::new(value), pos });
            }
        }

        loop {
            let (op, left_bp, right_bp) = match self.peek() {
                TokenKind::OrOr => (BinOp::Or, 1, 2),
                TokenKind::AndAnd => (BinOp::And, 3, 4),
                TokenKind::EqEq => (BinOp::Eq, 5, 6),
                TokenKind::Ne => (BinOp::Ne, 5, 6),
                TokenKind::Lt => (BinOp::Lt, 5, 6),
                TokenKind::Le => (BinOp::Le, 5, 6),
                TokenKind::Gt => (BinOp::Gt, 5, 6),
                TokenKind::Ge => (BinOp::Ge, 5, 6),
                TokenKind::Plus => (BinOp::Add, 7, 8),
                TokenKind::Minus => (BinOp::Sub, 7, 8),
                TokenKind::Star => (BinOp::Mul, 9, 10),
                TokenKind::Slash => (BinOp::Div, 9, 10),
                TokenKind::Percent => (BinOp::Rem, 9, 10),
                TokenKind::Caret => (BinOp::Pow, 14, 13),
                _ => break,
            };
            if left_bp < min_bp {
                break;
            }
            let pos = self.peek_pos();
            self.advance();
            let rhs = self.parse_expr(right_bp)?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), pos };
        }

        Ok(lhs)
    }

    fn parse_nud(&mut self) -> Result<Expr> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Int(v) => Ok(Expr::Int(v, tok.pos)),
            TokenKind::Float(v) => Ok(Expr::Float(v, tok.pos)),
            TokenKind::Str(s) => Ok(Expr::Str(s, tok.pos)),
            TokenKind::True => Ok(Expr::Bool(true, tok.pos)),
            TokenKind::False => Ok(Expr::Bool(false, tok.pos)),
            TokenKind::Ident(name) => {
                if matches!(self.peek(), TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr(0)?);
                            if matches!(self.peek(), TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.eat(&TokenKind::RParen)?;
                    Ok(Expr::Call { name, args, pos: tok.pos })
                } else {
                    Ok(Expr::Var(name, tok.pos))
                }
            }
            TokenKind::LParen => {
                let e = self.parse_expr(0)?;
                self.eat(&TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::Minus => {
                let rhs = self.parse_expr(11)?;
                Ok(Expr::Unary { op: UnOp::Neg, rhs: Box::new(rhs), pos: tok.pos })
            }
            TokenKind::Bang => {
                let rhs = self.parse_expr(11)?;
                Ok(Expr::Unary { op: UnOp::Not, rhs: Box::new(rhs), pos: tok.pos })
            }
            other => Err(CalcError::SyntaxError {
                msg: format!("Ожидалось выражение, но встретилось {:?}", other),
                pos: tok.pos,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, BinOp};
    fn parse_expr(src: &str) -> Expr {
        let toks = crate::lexer::tokenize(src).unwrap();
        Parser::new(toks).parse_single_expr().unwrap()
    }
    #[test]
    fn precedence_mul_over_add() {
        match parse_expr("1 + 2 * 3") {
            Expr::Binary { op: BinOp::Add, rhs, .. } => assert!(matches!(*rhs, Expr::Binary { op: BinOp::Mul, .. })),
            _ => panic!("ожидался Add на вершине"),
        }
    }
    #[test]
    fn pow_is_right_assoc() {
        match parse_expr("2 ^ 3 ^ 2") {
            Expr::Binary { op: BinOp::Pow, rhs, .. } => assert!(matches!(*rhs, Expr::Binary { op: BinOp::Pow, .. })),
            _ => panic!(),
        }
    }
    #[test]
    fn call_with_args() {
        match parse_expr("Max(1, 2, 3)") {
            Expr::Call { name, args, .. } => { assert_eq!(name, "Max"); assert_eq!(args.len(), 3); }
            _ => panic!(),
        }
    }
    #[test]
    fn unary_minus_parses() { let _ = parse_expr("-2 ^ 2"); }
    #[test]
    fn assignment_parses() {
        match parse_expr("x = 5") {
            Expr::Assign { name, .. } => assert_eq!(name, "x"),
            _ => panic!(),
        }
    }
    #[test]
    fn program_statements() {
        let toks = crate::lexer::tokenize("fn sq(x) = x*x; while (x > 0) { x = x - 1 }; repeat 3 { x = x + 1 }").unwrap();
        let stmts = Parser::new(toks).parse_program().unwrap();
        assert_eq!(stmts.len(), 3);
        assert!(matches!(stmts[0], crate::ast::Stmt::FnDef { .. }));
        assert!(matches!(stmts[1], crate::ast::Stmt::While { .. }));
        assert!(matches!(stmts[2], crate::ast::Stmt::Repeat { .. }));
    }
}
