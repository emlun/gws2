use git2::Commit;
use std::collections::LinkedList;

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
        self.ancestors().any(|ancestor| ancestor.id() == other.id())
    }
}

impl<'repo> WithAncestors<'repo> for Commit<'repo> {
    fn ancestors(&self) -> Ancestors<'repo> {
        Ancestors {
            queue: self.parents().collect(),
        }
    }
}
