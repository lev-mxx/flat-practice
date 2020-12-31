pub use BoolExpr::*;
pub use GraphExpr::*;
pub use ListExpr::*;
pub use ObjectExpr::*;
pub use Pattern::*;
pub use Script::*;
pub use SimpleGraph::*;
pub use Source::*;
pub use Statement::*;
pub use Vertices::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Script {
    Sequence(Vec<Statement>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Connect(Vec<String>),
    Define(String, Pattern),
    Get(ObjectExpr, Source),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    Apply(Pattern, GraphExpr),
    Direct(GraphExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum GraphExpr {
    Intersection(Vec<SimpleGraph>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimpleGraph {
    GraphName(String),
    WithEnds {
        initials: Vertices,
        finals: Vertices,
        graph: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Vertices {
    Set(Vec<usize>),
    Range { from: usize, to: usize },
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
    Filter(Box<ListExpr>, BoolExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BoolExpr {
    LabelIs(String),
    BeginIs(VertexVariant),
    EndIs(VertexVariant),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum VertexVariant {
    Initial,
    Final,
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
