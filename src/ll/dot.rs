use crate::ll::algo::{Node, Child};
use crate::syntax::dot::{ToDotG, DotContextG};

pub trait Printer<T> {
    fn nonterminal(&self, code: usize) -> String;

    fn terminal(&self, code: usize) -> String;

    fn value(&self, code: &T) -> String;
}

impl<'a, T: 'a> ToDotG<'a> for Node<T> {
    type Ctx = &'a dyn Printer<T>;

    fn to_dot_g(&self, ctx: &mut DotContextG<'a, Self::Ctx>) -> usize {
        ctx.fold_op(&format!("<{}>", ctx.t.nonterminal(self.nonterminal)), &self.children)
    }
}

impl<'a, T: 'a> ToDotG<'a> for Child<T> {
    type Ctx = &'a dyn Printer<T>;

    fn to_dot_g(&self, ctx: &mut DotContextG<'a, Self::Ctx>) -> usize {
        match self {
            Child::Value(terminal, t) => ctx.vertex(&format!("{} = {}", ctx.t.terminal(*terminal), ctx.t.value(t))),
            Child::Nonterminal(node) => node.to_dot_g(ctx),
            Child::Terminal(terminal) => ctx.vertex(&format!("{}", ctx.t.terminal(*terminal))),
        }
    }
}
