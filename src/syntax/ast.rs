
#[derive(Debug, PartialEq, Eq)]
pub enum Script {
    Sequence(Vec<Statement>),
}
pub use Script::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Connect(String),
    Define(String, Pattern),
    Get(ObjectExpr, GraphExpr),
}
pub use Statement::*;

#[derive(Debug, PartialEq, Eq)]
pub enum GraphExpr {
    Intersection(Box<GraphExpr>, Box<GraphExpr>),
    Query(Pattern),
    Graph { name: String },
    WithEnds { initials: Vertices, finals: Vertices, graph: Box<GraphExpr> },
}
pub use GraphExpr::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Vertices {
    Set(Vec<u64>),
    Range { from: u64, to: u64 },
    EmptySet,
}
pub use Vertices::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectExpr {
    Count(ListExpr),
    List(ListExpr),
}
pub use ObjectExpr::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ListExpr {
    Edges,
    Filter(Box<ListExpr>, Condition),
}
pub use ListExpr::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Condition {
    Cond(String, String, String, BoolExpr),
}
pub use Condition::*;

#[derive(Debug, PartialEq, Eq)]
pub enum BoolExpr {
    Is(String, String),
    IsStart(String),
    IsFinal(String),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
}
pub use BoolExpr::*;

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
pub use Pattern::*;

