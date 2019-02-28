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

        loop {
            match (self_ancestors.next(), other_ancestors.next()) {
                (Some(sa), Some(oa)) => {
                    if sa.id() == other_id {
                        return true;
                    } else if ancestors_seen.contains(&sa.id()) || ancestors_seen.contains(&oa.id())
                    {
                        return false;
                    } else {
                        ancestors_seen.insert(sa.id());
                        ancestors_seen.insert(oa.id());
                    }
                }
                (Some(sa), None) => {
                    if sa.id() == other_id {
                        return true;
                    } else if ancestors_seen.contains(&sa.id()) {
                        return false;
                    } else {
                        ancestors_seen.insert(sa.id());
                    }
                }
                (None, Some(oa)) => {
                    if ancestors_seen.contains(&oa.id()) {
                        return false;
                    } else {
                        ancestors_seen.insert(oa.id());
                    }
                }
                (None, None) => return false,
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
