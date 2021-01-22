use super::ast::*;

pub fn to_dot(s: &Script) -> String {
    let mut ctx = DotContext {
        buffer: String::from("digraph script {\n"),
        last_id: 0,
        t: &()
    };
    s.to_dot(&mut ctx);
    ctx.buffer.push('}');
    ctx.buffer
}

impl ToDot for Script {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Sequence(stmts) => {
                let id = ctx.vertex("sequence");
                for p in stmts {
                    let p_id = p.to_dot(ctx);
                    ctx.edge(id, p_id);
                }
                id
            }
        }
    }
}

impl ToDot for Statement {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Connect(path) => {
                let mut name = String::new();
                let size = path.len();
                for i in 0..size - 1 {
                    name.push_str(path[i].as_str());
                    name.push('.');
                }
                name.push_str(path[size - 1].as_str());
                ctx.vertex(&format!("Connect to {}", name))
            }
            Define(name, pattern) => ctx.op(&format!("Define {}", name), pattern),
            Get(o, g) => ctx.binop("Get", o, g),
        }
    }
}

impl ToDot for Source {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Apply(pattern, graph) => ctx.binop("apply", pattern, graph),
            Direct(graph) => graph.to_dot(ctx),
        }
    }
}

impl ToDot for GraphExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Intersection(gs) => ctx.fold_op("&", gs),
        }
    }
}

impl ToDot for SimpleGraph {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            GraphName(name) => ctx.vertex(&format!("Graph '{}'", name)),
            WithEnds {
                graph,
                finals,
                initials,
            } => {
                let g_id = ctx.vertex(&format!("Graph '{}'", graph));
                let f_id = ctx.op("Finals", finals);
                let s_id = initials.to_dot(ctx);
                let id = ctx.vertex("With ends");
                ctx.edge(id, g_id);
                ctx.edge(id, f_id);
                ctx.edge(id, s_id);
                id
            }
        }
    }
}

impl ToDot for Vertices {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Set(set) => ctx.vertex(&format!("{:?}", set)),
            Range { from, to } => ctx.vertex(&format!("[{}..{}]", from, to)),
            EmptySet => ctx.vertex("[]"),
        }
    }
}

impl ToDot for ObjectExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Count(l) => ctx.op("Count", l),
            List(l) => l.to_dot(ctx),
        }
    }
}

impl ToDot for ListExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Edges => ctx.vertex("Edges"),
            Filter(list, cond) => ctx.binop("Filter", list.as_ref(), cond),
        }
    }
}

impl ToDot for BoolExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            LabelIs(s) => ctx.vertex(&format!("label is '{}'", s)),
            BeginIs(s) => ctx.vertex(&format!("begin is {:?}", s)),
            EndIs(s) => ctx.vertex(&format!("end is {:?}", s)),
            And(b1, b2) => ctx.binop("and", b1.as_ref(), b2.as_ref()),
            Or(b1, b2) => ctx.binop("or", b1.as_ref(), b2.as_ref()),
            Not(b) => ctx.op("not", b.as_ref()),
        }
    }
}

impl ToDot for Pattern {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Term(str) => ctx.vertex(&format!("Terminal '{}'", str)),
            Var(str) => ctx.vertex(&format!("Variable '{}'", str)),
            Star(pattern) => ctx.op("*", pattern.as_ref()),
            Plus(pattern) => ctx.op("+", pattern.as_ref()),
            Maybe(pattern) => ctx.op("?", pattern.as_ref()),
            Alt(p1, p2) => ctx.binop("|", p1.as_ref(), p2.as_ref()),
            Seq(vec) => ctx.fold_op(".", vec),
        }
    }
}

pub struct DotContextG<'a, T> {
    pub buffer: String,
    pub last_id: usize,
    pub t: &'a T,
}

impl<'a, T> DotContextG<'a, T> {
    pub fn vertex(&mut self, label: &str) -> usize {
        let id = self.last_id;
        self.last_id += 1;
        self.buffer
            .push_str(format!("\t{}[label=\"{}\"]\n", id, label).as_str());
        id
    }

    pub fn edge(&mut self, from: usize, to: usize) {
        self.buffer
            .push_str(format!("\t{}->{}\n", from, to).as_str());
    }

    pub fn binop(&mut self, op: &str, o1: &dyn ToDotG<'a, Ctx = T>, o2: &dyn ToDotG<'a, Ctx = T>) -> usize {
        let id = self.vertex(op);
        let o1_id = o1.to_dot_g(self);
        let o2_id = o2.to_dot_g(self);
        self.edge(id, o1_id);
        self.edge(id, o2_id);
        id
    }

    pub fn op(&mut self, op: &str, o: &dyn ToDotG<'a, Ctx = T>) -> usize {
        let id = self.vertex(op);
        let o_id = o.to_dot_g(self);
        self.edge(id, o_id);
        id
    }

    pub fn fold_op(&mut self, op: &str, os: &Vec<impl ToDotG<'a, Ctx = T>>) -> usize {
        let id = self.vertex(op);
        for o in os {
            let p_id = o.to_dot_g(self);
            self.edge(id, p_id);
        }
        id
    }
}

type DotContext = DotContextG<'static, ()>;

pub trait ToDot {
    fn to_dot(&self, ctx: &mut DotContext) -> usize;
}

impl<T: ToDot> ToDotG<'static> for T {
    type Ctx = ();

    fn to_dot_g(&self, ctx: &mut DotContextG<'static, ()>) -> usize {
        self.to_dot(ctx)
    }
}

pub trait ToDotG<'a> {
    type Ctx;

    fn to_dot_g(&self, ctx: &mut DotContextG<'a, Self::Ctx>) -> usize;
}
