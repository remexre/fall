use std::collections::HashMap;
use std::sync::Arc;

use futures::{
    prelude::*,
    stream::{empty, iter_ok, once, Empty},
};

use unify::{unify, Subst};
use util::box_stream;
use {Clause, Lit, Rules, Term};

/// An execution environment.
pub struct Env<F> {
    // rules is indexed by functor to make lookup faster.
    rules: Arc<HashMap<(Arc<str>, usize), Vec<Clause>>>,

    // An external function for resolving predicates.
    external: Arc<F>,
}

// The unit type argument is just a placeholder.
impl Env<()> {
    /// Creates an env without an external resolver.
    #[allow(dead_code)]
    pub fn new_self_contained<E: 'static + Send>(
        rules: &Rules,
    ) -> Env<for<'a> fn(&'a Lit) -> Empty<Subst, E>> {
        Env::new(rules, |_| empty())
    }
}

impl<E, F, S> Env<F>
where
    E: 'static + Send,
    F: 'static + for<'a> Fn(&'a Lit) -> S + Send + Sync,
    S: 'static + Stream<Item = Subst, Error = E> + Send,
{
    /// Create an Env.
    pub fn new(rules: &Rules, external: F) -> Env<F> {
        let mut internal = HashMap::<_, Vec<_>>::new();
        for rule in &rules.0 {
            internal
                .entry(rule.0.functor())
                .or_insert_with(Vec::new)
                .push(rule.clone());
        }

        Env {
            rules: Arc::new(internal),
            external: Arc::new(external),
        }
    }

    /// Tries to solve for the given literal.
    pub fn solve(&self, lit: Lit, depth: usize) -> impl Stream<Item = Subst, Error = E> + Send {
        if depth == 0 {
            box_stream(empty())
        } else {
            box_stream((self.external)(&lit).chain(self.solve_internal(lit, depth)))
        }
    }

    /// Tries to solve for multiple literals.
    pub fn solve_all(
        &self,
        mut goals: Vec<Lit>,
        depth: usize,
    ) -> Box<Stream<Item = Subst, Error = E> + Send> {
        if goals.is_empty() {
            box_stream(once(Ok(Subst::new())))
        } else {
            let head = goals.remove(0);
            let env = self.clone();
            box_stream(
                self.solve(head, depth)
                    .map(move |s| {
                        let tail = goals.iter().map(|l| s.apply_to_lit(l)).collect::<Vec<_>>();
                        env.solve_all(tail, depth).map(move |s2| s.merge(s2))
                    })
                    .flatten(),
            )
        }
    }

    fn solve_internal(
        &self,
        goal: Lit,
        depth: usize,
    ) -> Box<Stream<Item = Subst, Error = E> + Send> {
        lazy_static! {
            static ref TRUE: Lit = Lit("true".into(), vec![]);
        }

        if goal == *TRUE {
            box_stream(once(Ok(Subst::new())))
        } else {
            let rules = self
                .rules
                .get(&goal.functor())
                .map(|v| v as &[Clause])
                .unwrap_or_default();
            let goal = Arc::new(Term::Lit(goal));
            let rules = rules
                .iter()
                .map(|c| c.freshen())
                .filter_map(|Clause(h, body)| {
                    let head = Arc::new(Term::Lit(h));
                    let subst = unify(head, goal.clone())?;
                    let body = body
                        .iter()
                        .map(|l| subst.apply_to_lit(l))
                        .collect::<Vec<_>>();
                    Some((subst, body))
                })
                .collect::<Vec<_>>();
            let env = self.clone();
            box_stream(
                iter_ok(rules)
                    .map(move |(subst, body)| {
                        env.solve_all(body, depth).map(move |s| subst.merge(s))
                    })
                    .flatten(),
            )
        }
    }
}

impl<F> Clone for Env<F> {
    fn clone(&self) -> Env<F> {
        Env {
            rules: self.rules.clone(),
            external: self.external.clone(),
        }
    }
}
