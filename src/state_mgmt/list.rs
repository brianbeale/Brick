use std::rc::Rc;

pub enum ListChange<T> {
    Push(Rc<T>),
    Remove(Rc<T>),
    Clear,
}

pub trait ListObserver<T> {
    fn notify_list(&mut self, change: &ListChange<T>);
}

pub struct List<T> {
    items: Vec<Rc<T>>,
    observers: Vec<Box<dyn ListObserver<T>>>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            items: Vec::new(),
            observers: Vec::new(),
        }
    }

    pub fn push(&mut self, item: T) -> Rc<T> {
        let rc = Rc::new(item);
        self.items.push(Rc::clone(&rc));
        let change = ListChange::Push(Rc::clone(&rc));
        for obs in &mut self.observers {
            obs.notify_list(&change);
        }
        rc
    }

    pub fn remove(&mut self, target: &Rc<T>) {
        let pos = self.items.iter().position(|item| Rc::ptr_eq(item, target));
        if let Some(i) = pos {
            self.items.remove(i);
            let change = ListChange::Remove(Rc::clone(target));
            for obs in &mut self.observers {
                obs.notify_list(&change);
            }
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        for obs in &mut self.observers {
            obs.notify_list(&ListChange::Clear);
        }
    }

    pub fn items(&self) -> &[Rc<T>] {
        &self.items
    }

    pub fn add_observer(&mut self, obs: Box<dyn ListObserver<T>>) {
        self.observers.push(obs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_adds_item() {
        let mut ls: List<i32> = List::new();
        ls.push(1);
        assert_eq!(ls.items().len(), 1);
    }

    #[test]
    fn remove_by_pointer() {
        let mut ls: List<i32> = List::new();
        let rc = ls.push(42);
        assert_eq!(ls.items().len(), 1);
        ls.remove(&rc);
        assert_eq!(ls.items().len(), 0);
    }

    #[test]
    fn remove_doppelganger_leaves_other() {
        let mut ls: List<i32> = List::new();
        let a = ls.push(7);
        let b = ls.push(7);
        ls.remove(&a);
        assert_eq!(ls.items().len(), 1);
        assert!(Rc::ptr_eq(&ls.items()[0], &b));
    }

    #[test]
    fn clear_empties_list() {
        let mut ls: List<i32> = List::new();
        ls.push(1);
        ls.push(2);
        ls.clear();
        assert_eq!(ls.items().len(), 0);
    }

    struct CountingObserver {
        pushes: usize,
        removes: usize,
    }
    impl ListObserver<i32> for CountingObserver {
        fn notify_list(&mut self, change: &ListChange<i32>) {
            match change {
                ListChange::Push(_) => self.pushes += 1,
                ListChange::Remove(_) => self.removes += 1,
                ListChange::Clear => {}
            }
        }
    }

    #[test]
    fn observer_receives_push_and_remove() {
        let mut ls: List<i32> = List::new();
        ls.add_observer(Box::new(CountingObserver {
            pushes: 0,
            removes: 0,
        }));
        let rc = ls.push(1);
        ls.remove(&rc);
        assert_eq!(ls.items().len(), 0);
    }
}
