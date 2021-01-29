use serde::{Serialize, Deserialize};
use crate::ll::algo::{Table, Node};
use crate::ll::chars::CharInfo;
use anyhow::Result;
use crate::ll::dot::Printer;
use crate::syntax::dot::{DotContextG, ToDotG};

pub mod chars;
mod algo;
mod dot;

pub fn build_table(text: &str) -> Result<Data> {
    let (info, cfg) = chars::parse_cfg(text)?;
    let table = Table::build(cfg)?;
    Ok(Data {
        info,
        table
    })
}

pub fn build_ast(data: &Data, text: &str) -> Result<Node<()>> {
    let mut tokens = chars::CharTokenizer {
        chars: text.chars(),
        current: None
    };
    data.table.build_ast(&mut tokens)
}

pub fn to_dot(data: &Data, node: &Node<()>) -> String {
    let mut ctx = DotContextG::<&dyn Printer<()>> {
        buffer: String::from("digraph script {\n"),
        last_id: 0,
        t: &(data as &dyn Printer<()>)
    };

    node.to_dot_g(&mut ctx);
    ctx.buffer.push('}');
    ctx.buffer
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    info: CharInfo,
    table: Table,
}

impl Printer<()> for Data {
    fn nonterminal(&self, code: usize) -> String {
        self.info.nonterminals[code].to_string()
    }

    fn terminal(&self, code: usize) -> String {
        unsafe { char::from_u32_unchecked(code as u32) }.to_string()
    }

    fn value(&self, _: &()) -> String {
        unimplemented!()
    }
}
