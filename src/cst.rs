use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use ast;
use util::gensym;

/// A convenient alias for a LALRPOP parse error.
pub type ParseError = ::lalrpop_util::ParseError<usize, (usize, String), &'static str>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rules(pub Vec<Clause>);

impl Rules {
    pub fn to_ast(self, atoms: &mut HashSet<Arc<str>>) -> ast::Rules {
        let mut rules = Vec::new();
        for clause in self.0 {
            rules.push(clause.to_ast(atoms));
        }
        ast::Rules(rules)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clause(pub Lit, pub Vec<Lit>);

impl Clause {
    pub fn to_ast(self, atoms: &mut HashSet<Arc<str>>) -> ast::Clause {
        let Clause(h, b) = self;
        let mut scope = HashMap::new();

        let head = h.to_ast(atoms, &mut scope);
        let mut body = Vec::new();
        for lit in b {
            body.push(lit.to_ast(atoms, &mut scope));
        }
        ast::Clause(head, body)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Term {
    Any,
    Lit(Lit),
    Num(u32),
    Var(String),
}

impl Term {
    pub fn to_ast(
        self,
        atoms: &mut HashSet<Arc<str>>,
        scope: &mut HashMap<String, usize>,
    ) -> Arc<ast::Term> {
        Arc::new(match self {
            Term::Any => ast::Term::Var(gensym()),
            Term::Lit(lit) => ast::Term::Lit(lit.to_ast(atoms, scope)),
            Term::Num(n) => ast::Term::Num(n),
            Term::Var(name) => {
                let id = scope.entry(name).or_insert_with(gensym);
                ast::Term::Var(*id)
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lit(pub String, pub Vec<Term>);

impl Lit {
    pub fn to_ast(
        self,
        atoms: &mut HashSet<Arc<str>>,
        scope: &mut HashMap<String, usize>,
    ) -> ast::Lit {
        let Lit(n, a) = self;

        let name = atoms.get(&n as &str).cloned().unwrap_or_else(|| {
            let name: Arc<str> = Arc::from(n);
            atoms.insert(name.clone());
            name
        });

        let mut args = Vec::new();
        for arg in a {
            args.push(arg.to_ast(atoms, scope));
        }
        ast::Lit(name, args)
    }
}

macro_rules! generate_fromstr {
    ($name:ident $parser:ident) => {
        impl FromStr for $name {
            type Err = ParseError;

            fn from_str(s: &str) -> Result<$name, ParseError> {
                use grammar::Token;
                ::grammar::$parser::new()
                    .parse(s).map_err(|e| e.map_token(|Token(n, s)| (n, s.to_string())))
            }
        }
    };
    ($(($name:ident, $parser:ident)),*) => { $(generate_fromstr!($name $parser);)* };
}

generate_fromstr![
    (Clause, ClauseParser),
    (Lit, LitParser),
    (Rules, RulesParser),
    (Term, TermParser)
];
