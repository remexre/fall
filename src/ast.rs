use std::collections::HashMap;
#[cfg(feature = "parser")]
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(feature = "parser")]
use std::fs::File;
#[cfg(feature = "parser")]
use std::io::Read;
#[cfg(feature = "parser")]
use std::path::Path;
#[cfg(feature = "parser")]
use std::str::FromStr;
use std::sync::Arc;

use regex::Regex;

#[cfg(feature = "parser")]
use cst;
#[cfg(feature = "parser")]
use errors::LoadError;
use util::gensym;

/// A collection of rules or facts (as clauses).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Rules(pub Vec<Clause>);

impl Rules {
    /// Loads from a file.
    #[cfg(feature = "parser")]
    pub fn load_from(path: impl AsRef<Path>) -> Result<Rules, LoadError> {
        let src = {
            let mut f = File::open(path).map_err(LoadError::Io)?;
            let mut buf = String::new();
            f.read_to_string(&mut buf).map_err(LoadError::Io)?;
            buf
        };
        src.parse().map_err(LoadError::Parse)
    }
}

#[cfg(feature = "parser")]
impl FromStr for Rules {
    type Err = <cst::Rules as FromStr>::Err;
    fn from_str(src: &str) -> Result<Rules, Self::Err> {
        let cst = src.parse::<cst::Rules>()?;
        let mut atoms = HashSet::new();
        Ok(cst.to_ast(&mut atoms))
    }
}

/// A single rule or fact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Clause(pub Lit, pub Vec<Lit>);

impl Clause {
    /// Replaces all variables in the clause with fresh ones.
    pub fn freshen(&self) -> Clause {
        let Clause(ref head, ref body) = *self;
        let mut vars = HashMap::new();
        let head = head.freshen_helper(&mut vars);
        let body = body
            .iter()
            .map(|lit| lit.freshen_helper(&mut vars))
            .collect::<Vec<_>>();
        Clause(head, body)
    }
}

impl Display for Clause {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let Clause(ref head, ref body) = *self;

        write!(fmt, "{}", head)?;
        let mut first = true;
        for lit in body {
            let prefix = if first {
                first = false;
                " :- "
            } else {
                ", "
            };
            write!(fmt, "{}{}", prefix, lit)?;
        }
        write!(fmt, ".")
    }
}

/// A term, e.g. `even(X)`, `30`, `foo`, or `Bar`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Term {
    /// A literal value, e.g. `foo`, `bar(1, 2)`, or `baz(X, a, X)`.
    Lit(Lit),

    /// A numeric literal, e.g. `0`, `42`, or `137`.
    Num(u32),

    /// A variable. Each variable is globally unified against, so the actual resolution procedure
    /// will "freshen" a clause before running it by replacing its variables with fresh ones.
    Var(usize),
}

impl Term {
    fn freshen_helper(&self, vars: &mut HashMap<usize, usize>) -> Arc<Term> {
        Arc::new(match *self {
            Term::Lit(ref l) => Term::Lit(l.freshen_helper(vars)),
            Term::Num(n) => Term::Num(n),
            Term::Var(v) => Term::Var(*vars.entry(v).or_insert_with(gensym)),
        })
    }

    /// Returns a unique `Var`.
    pub fn gensym() -> Arc<Term> {
        Arc::new(Term::Var(gensym()))
    }
}

impl Display for Term {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Term::Lit(ref l) => write!(fmt, "{}", l),
            Term::Num(n) => write!(fmt, "{}", n),
            Term::Var(v) => write!(fmt, "_{}", v),
        }
    }
}

/// A literal value, e.g. `foo`, `bar(1, 2)`, or `baz(X, a, X)`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Lit(pub Arc<str>, pub Vec<Arc<Term>>);

impl Lit {
    fn freshen_helper(&self, vars: &mut HashMap<usize, usize>) -> Lit {
        let Lit(ref name, ref args) = *self;
        let args = args
            .iter()
            .map(|a| a.freshen_helper(vars))
            .collect::<Vec<_>>();
        Lit(name.clone(), args)
    }

    /// Returns the name and arity of the literal.
    pub fn functor(&self) -> (Arc<str>, usize) {
        (self.0.clone(), self.1.len())
    }

    /// Returns the name and arity of the literal.
    pub fn functor_b(&self) -> (&str, usize) {
        (&*self.0, self.1.len())
    }
}

impl Display for Lit {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        lazy_static! {
            static ref UNQUOTED_ATOM: Regex = Regex::new("[a-z.][A-Za-z_.]*").unwrap();
        }

        let Lit(ref name, ref args) = *self;
        if UNQUOTED_ATOM.is_match(name) {
            write!(fmt, "{}", name)?;
        } else if name.contains('\'') {
            write!(fmt, "\"{}\"", name)?;
        } else {
            write!(fmt, "'{}'", name)?;
        }

        if !args.is_empty() {
            let mut first = true;
            for arg in args {
                let prefix = if first {
                    first = false;
                    "("
                } else {
                    ", "
                };
                write!(fmt, "{}{}", prefix, arg)?;
            }
            write!(fmt, ")")?;
        }

        Ok(())
    }
}
