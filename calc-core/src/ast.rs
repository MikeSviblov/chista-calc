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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_expr() {
        let e = Expr::Binary { op: BinOp::Add, lhs: Box::new(Expr::Int(1, 0)), rhs: Box::new(Expr::Int(2, 2)), pos: 1 };
        assert!(matches!(e, Expr::Binary { op: BinOp::Add, .. }));
    }
}
