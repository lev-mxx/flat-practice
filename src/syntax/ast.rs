pub use BoolExpr::*;
pub use Condition::*;
pub use GraphExpr::*;
pub use ListExpr::*;
pub use ObjectExpr::*;
pub use Pattern::*;
pub use Script::*;
pub use Statement::*;
pub use Vertices::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Script {
    Sequence(Vec<Statement>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Connect(String),
    Define(String, Pattern),
    Get(ObjectExpr, GraphExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum GraphExpr {
    Intersection(Box<GraphExpr>, Box<GraphExpr>),
    Query(Pattern),
    Graph { name: String },
    WithEnds { initials: Vertices, finals: Vertices, graph: Box<GraphExpr> },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Vertices {
    Set(Vec<u64>),
    Range { from: u64, to: u64 },
    EmptySet,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectExpr {
    Count(ListExpr),
    List(ListExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListExpr {
    Edges,
    Filter(Box<ListExpr>, Condition),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Condition {
    Cond(String, String, String, BoolExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BoolExpr {
    Is(String, String),
    IsStart(String),
    IsFinal(String),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Pattern {
    Term(String),
    Var(String),
    Star(Box<Pattern>),
    Plus(Box<Pattern>),
    Maybe(Box<Pattern>),
    Alt(Box<Pattern>, Box<Pattern>),
    Seq(Vec<Pattern>),
}
