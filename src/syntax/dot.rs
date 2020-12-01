use super::ast::*;

pub fn to_dot(s: &Script) -> String {
    let mut ctx = DotContext {
        buffer: String::from("digraph script {\n"),
        last_id: 0
    };
    s.to_dot(&mut ctx);
    ctx.buffer.push('}');
    ctx.buffer
}

impl ToDot for Script {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Sequence(stmts) => {
                let id = ctx.vertex(&"sequence");
                for p in stmts {
                    let p_id = p.to_dot(ctx);
                    ctx.edge(id, p_id);
                }
                id
            },
        }
    }
}

impl ToDot for Statement {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Connect(db) => ctx.vertex(&format!("Connect to {}", db)),
            Define(name, pattern) => ctx.op(&format!("Define {}", name), pattern),
            Get(o, g) => ctx.binop(&"Get", o, g),
        }
    }
}

impl ToDot for GraphExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Intersection(g1, g2) => ctx.binop(&"&", g1.as_ref(), g2.as_ref()),
            Query(p) => p.to_dot(ctx),
            Graph { name } => ctx.vertex(name),
            WithEnds { graph, finals, initials } => {
                let g_id = ctx.op(&"Graph", graph.as_ref());
                let f_id = ctx.op(&"Finals", finals);
                let s_id = initials.to_dot(ctx);
                let id = ctx.vertex(&"With ends");
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
            EmptySet => ctx.vertex(&"[]"),
        }
    }
}

impl ToDot for ObjectExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Count(l) => ctx.op(&"Count", l),
            List(l) => l.to_dot(ctx),
        }
    }
}

impl ToDot for ListExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Edges => ctx.vertex(&"Edges"),
            Filter(list, cond) => ctx.binop(&"Filter", list.as_ref(), cond),
        }
    }
}

impl ToDot for Condition {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Cond(from, to, label, b) => {
                ctx.op(&format!("{} {} {} satisfy", from, to, label), b)
            }
        }
    }
}

impl ToDot for BoolExpr {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Is(s1, s2) => { ctx.vertex(&format!("{} is '{}'", s1, s2)) }
            IsStart(s) => { ctx.vertex(&format!("{} is start", s)) }
            IsFinal(s) => { ctx.vertex(&format!("{} is final", s)) }
            And(b1, b2) => ctx.binop(&"and", b1.as_ref(), b2.as_ref()),
            Or(b1, b2) => ctx.binop(&"or", b1.as_ref(), b2.as_ref()),
            Not(b) => ctx.op(&"not", b.as_ref()),
        }
    }
}

impl ToDot for Pattern {
    fn to_dot(&self, ctx: &mut DotContext) -> usize {
        match self {
            Term(str) => {
                ctx.vertex(&format!("Terminal '{}'", str))
            }
            Var(str) => {
                ctx.vertex(&format!("Variable '{}'", str))
            }
            Star(pattern) => ctx.op(&"*", pattern.as_ref()),
            Plus(pattern) => ctx.op(&"+", pattern.as_ref()),
            Maybe(pattern) => ctx.op(&"?", pattern.as_ref()),
            Alt(p1, p2) => ctx.binop(&"|", p1.as_ref(), p2.as_ref()),
            Seq(vec) => {
                let id = ctx.vertex(&".");
                for p in vec {
                    let p_id = p.to_dot(ctx);
                    ctx.edge(id, p_id);
                }
                id
            }
        }
    }
}

struct DotContext {
    buffer: String,
    last_id: usize,
}

impl DotContext {
    pub fn vertex(&mut self, label: &dyn AsRef<str>) -> usize {
        let id = self.last_id;
        self.last_id += 1;
        self.buffer.push_str(format!("\t{}[label=\"{}\"]\n", id, label.as_ref()).as_str());
        id
    }

    pub fn edge(&mut self, from: usize, to: usize) {
        self.buffer.push_str(format!("\t{}->{}\n", from, to).as_str());
    }

    pub fn binop(&mut self, op: &dyn AsRef<str>, o1: &dyn ToDot, o2: &dyn ToDot) -> usize {
        let id = self.vertex(op);
        let o1_id = o1.to_dot(self);
        let o2_id = o2.to_dot(self);
        self.edge(id, o1_id);
        self.edge(id, o2_id);
        id
    }

    pub fn op(&mut self, op: &dyn AsRef<str>, o: &dyn ToDot) -> usize {
        let id = self.vertex(op);
        let o_id = o.to_dot(self);
        self.edge(id, o_id);
        id
    }
}

trait ToDot {
    fn to_dot(&self, ctx: &mut DotContext) -> usize;
}
