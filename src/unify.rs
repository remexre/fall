use std::collections::HashMap;
use std::sync::Arc;

use ast::{Lit, Term};

/// A substitution.
#[derive(Clone, Debug, Default)]
pub struct Subst(HashMap<usize, Arc<Term>>);

impl Subst {
    /// Creates a new, empty substitution.
    pub fn new() -> Subst {
        Subst(HashMap::new())
    }

    /// Applies a substitution to a term.
    pub fn apply_to_term<'a>(&'a self, mut term: &'a Term) -> Arc<Term> {
        loop {
            match *term {
                Term::Lit(ref l) => break Arc::new(Term::Lit(self.apply_to_lit(l))),
                Term::Num(n) => break Arc::new(Term::Num(n)),
                Term::Var(n) => {
                    if let Some(v) = self.get(n) {
                        term = &*v;
                    } else {
                        break Arc::new(Term::Var(n));
                    }
                }
            }
        }
    }

    /// Applies a substitution to a lit.
    pub fn apply_to_lit(&self, lit: &Lit) -> Lit {
        let Lit(ref n, ref a) = *lit;
        let a = a.iter().map(|t| self.apply_to_term(&*t)).collect();
        Lit(n.clone(), a)
    }

    /// Gets the replacement for the given variable index, if any.
    pub fn get(&self, k: usize) -> Option<&Arc<Term>> {
        self.0.get(&k)
    }

    /// Merges with another substitution. The substitution on the left should be the "older" one.
    pub fn merge(&self, other: Subst) -> Subst {
        let mut new = self.clone();
        for (k, v) in other.0 {
            new.push(k, v);
        }
        new
    }

    /// Adds a binding to the substitution.
    pub fn push(&mut self, k: usize, v: Arc<Term>) {
        let mut subst = HashMap::new();
        subst.insert(k, v.clone());
        let subst = Subst(subst);
        for v in self.0.values_mut() {
            *v = subst.apply_to_term(&*v);
        }
        assert!(!self.0.contains_key(&k));
        self.0.insert(k, v);
    }
}

/// Unifies two terms, returning a list of substitutions from variable IDs to terms.
pub fn unify(l: Arc<Term>, r: Arc<Term>) -> Option<Subst> {
    let mut subst = Subst::new();
    unify_helper(l, r, &mut subst)?;
    Some(subst)
}

fn unify_helper(l: Arc<Term>, r: Arc<Term>, subst: &mut Subst) -> Option<()> {
    match (&*l, &*r) {
        (&Term::Var(l), _) => {
            subst.push(l, r.clone());
            Some(())
        }
        (_, &Term::Var(r)) => {
            subst.push(r, l.clone());
            Some(())
        }
        (Term::Lit(l), Term::Lit(r)) => {
            if l.functor() == r.functor() {
                for (l, r) in l.1.iter().zip(r.1.iter()) {
                    let l = subst.apply_to_term(l);
                    let r = subst.apply_to_term(r);
                    unify_helper(l.clone(), r.clone(), subst)?;
                }
                Some(())
            } else {
                None
            }
        }
        (&Term::Num(l), &Term::Num(r)) => if l == r {
            Some(())
        } else {
            None
        },
        _ => None,
    }
}
