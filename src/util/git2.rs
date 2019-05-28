use std::collections::HashSet;
use std::collections::LinkedList;

use git2::Commit;
use git2::Oid;

pub struct Ancestors<'repo> {
    queue: LinkedList<Commit<'repo>>,
}

impl<'repo> Iterator for Ancestors<'repo> {
    type Item = Commit<'repo>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_front() {
            None => None,
            Some(next) => {
                self.queue.extend(next.parents());
                Some(next)
            }
        }
    }
}

pub trait WithAncestors<'repo> {
    fn ancestors(&self) -> Ancestors<'repo>;

    fn is_descendant_of(&self, other: &Commit<'repo>) -> bool {
        let other_id = other.id();
        let mut ancestors_seen: HashSet<Oid> = HashSet::new();
        let mut self_ancestors = self.ancestors();
        let mut other_ancestors = other.ancestors();
        let mut other_ancestors_seen: HashSet<Oid> = HashSet::new();

        // Walk backwards in history from both self and other, in parallel, one
        // commit at a time.
        loop {
            match (self_ancestors.next(), other_ancestors.next()) {
                (Some(sa), Some(oa)) => {
                    if sa.id() == other_id {
                        // If at any point we encounter `other` as an ancestor
                        // of `self`, then `other` clearly is an ancestor of
                        // `self`.
                        return true;
                    } else if other_ancestors_seen.contains(&sa.id())
                        || ancestors_seen.contains(&oa.id())
                    {
                        // If the current self-ancestor is also an ancestor of
                        // `other`, or if the current other-ancestor is also an
                        // ancestor of `self`, then `self` and `other` have a
                        // common ancestor and `self` cannot be a descendant of
                        // `other`.
                        return false;
                    } else {
                        ancestors_seen.insert(sa.id());
                        other_ancestors_seen.insert(oa.id());
                    }
                }
                (Some(sa), None) => {
                    if sa.id() == other_id {
                        // If at any point we encounter `other` as an ancestor
                        // of `self`, then `other` clearly is an ancestor of
                        // `self`.
                        return true;
                    } else if other_ancestors_seen.contains(&sa.id()) {
                        // If the current self-ancestor is also an ancestor of
                        // `other`, then `self` and `other` have a common
                        // ancestor and `self` cannot be a descendant of
                        // `other`.
                        return false;
                    } else {
                        ancestors_seen.insert(sa.id());
                    }
                }
                (None, _) => {
                    // If we run out of self-ancestors, then `other` clearly is
                    // not one of them.
                    return false;
                }
            }
        }
    }
}

impl<'repo> WithAncestors<'repo> for Commit<'repo> {
    fn ancestors(&self) -> Ancestors<'repo> {
        Ancestors {
            queue: self.parents().collect(),
        }
    }
}
