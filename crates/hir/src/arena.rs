use std::marker::PhantomData;
use std::ops::Index;

use string_interner::{DefaultSymbol, StringInterner};

use crate::amt::expr::Expr;
use crate::amt::Item;

#[derive(Default)]
pub struct Arena {
    items: Vec<Item>,
    exprs: Vec<Expr>,
    strings: StringInterner,
}

impl Arena {
    pub fn new() -> Self { Arena::default() }

    pub fn add_item(&mut self, item: Item) -> Idx<Item> {
        let idx = Idx::new(self.items.len());
        self.items.push(item);
        idx
    }

    pub fn add_expr(&mut self, expr: Expr) -> Idx<Expr> {
        let idx = Idx::new(self.exprs.len());
        self.exprs.push(expr);
        idx
    }

    pub fn add_string(&mut self, string: &str) -> DefaultSymbol {
        self.strings.get_or_intern(string)
    }
}

impl Index<Idx<Item>> for Arena {
    type Output = Item;

    fn index(&self, index: Idx<Item>) -> &Self::Output { &self.items[index.idx] }
}

impl Index<DefaultSymbol> for Arena {
    type Output = str;

    fn index(&self, index: DefaultSymbol) -> &Self::Output {
        self.strings.resolve(index).unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Idx<T> {
    idx: usize,
    _t: PhantomData<T>,
}

impl<T> Idx<T> {
    fn new(idx: usize) -> Self { Self { idx, _t: PhantomData } }
}
