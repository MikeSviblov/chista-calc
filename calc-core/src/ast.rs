#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i128, usize),
    Float(f64, usize),
    Str(String, usize),
    Bool(bool, usize),
    Var(String, usize),
    Unary { op: UnOp, rhs: Box<Expr>, pos: usize },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, pos: usize },
    Call { name: String, args: Vec<Expr>, pos: usize },
    Assign { name: String, value: Box<Expr>, pos: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    FnDef { name: String, params: Vec<String>, body: Expr, pos: usize },
    Alias { name: String, target: String, pos: usize },
    While { cond: Expr, body: Vec<Stmt>, pos: usize },
    Repeat { count: Expr, body: Vec<Stmt>, pos: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

pub fn expr_pos(e: &Expr) -> usize {
    match e {
        Expr::Int(_, pos)
        | Expr::Float(_, pos)
        | Expr::Str(_, pos)
        | Expr::Bool(_, pos)
        | Expr::Var(_, pos)
        | Expr::Unary { pos, .. }
        | Expr::Binary { pos, .. }
        | Expr::Call { pos, .. }
        | Expr::Assign { pos, .. } => *pos,
    }
}

pub fn stmt_pos(s: &Stmt) -> usize {
    match s {
        Stmt::Expr(e) => expr_pos(e),
        Stmt::FnDef { pos, .. }
        | Stmt::Alias { pos, .. }
        | Stmt::While { pos, .. }
        | Stmt::Repeat { pos, .. } => *pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_expr() {
        let e = Expr::Binary { op: BinOp::Add, lhs: Box::new(Expr::Int(1, 0)), rhs: Box::new(Expr::Int(2, 2)), pos: 1 };
        assert!(matches!(e, Expr::Binary { op: BinOp::Add, .. }));
    }
    #[test]
    fn expr_and_stmt_pos() {
        let e = Expr::Binary { op: BinOp::Add, lhs: Box::new(Expr::Int(1,3)), rhs: Box::new(Expr::Int(2,7)), pos: 5 };
        assert_eq!(expr_pos(&e), 5);
        assert_eq!(stmt_pos(&Stmt::Expr(Expr::Int(9, 42))), 42);
        assert_eq!(stmt_pos(&Stmt::FnDef { name: "f".into(), params: vec![], body: Expr::Int(1,0), pos: 11 }), 11);
    }
}
