use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;

pub trait Observer<T> {
    fn notify(&mut self, datum: &T);
}
pub trait Subject<T> {
    fn update(&mut self, new_datum: T);
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>);
    fn remove_observer(&mut self, name: &str);
}

pub struct State<T> {
    datum: T,
    observers: HashMap<String, Box<dyn Observer<T>>>,
}
impl<T> State<T> {
    pub fn new(datum: T) -> Self {
        State::<T> {
            datum,
            observers: HashMap::new(),
        }
    }
    fn refresh(&mut self) {
        for (_class_name, obs) in &mut self.observers {
            obs.notify(&self.datum);
        }
    }
}
impl<T> Subject<T> for State<T> {
    fn update(&mut self, new_datum: T) {
        self.datum = new_datum;
        self.refresh();
    }
    fn add_observer(&mut self, class_name: &str, obs: Box<dyn Observer<T>>) {
        self.observers.insert(class_name.to_string(), obs);
    }
    fn remove_observer(&mut self, class_name: &str) {
        self.observers.remove(class_name);
    }
}
impl<T: Display> Display for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.datum.to_string())
    }
}
impl<T> Deref for State<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.datum
    }
}

pub struct SpanObserver {
    class_name: String,
}
impl SpanObserver {
    pub fn new(class_name: &str) -> Self {
        SpanObserver {
            class_name: class_name.to_string(),
        }
    }
}
impl<T: Display> Observer<T> for SpanObserver {
    fn notify(&mut self, datum: &T) {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_elements_by_class_name(&self.class_name)
            .item(0)
            .unwrap()
            .set_text_content(Some(&datum.to_string()));
    }
}

#[macro_export]
macro_rules! set {
    ( $subject:expr => + $change:expr ) => {
        $subject.update(*$subject + $change);
    };
    ( $subject:expr => - $change:expr ) => {
        $subject.update(*$subject - $change);
    };
    ( $subject:expr => * $change:expr ) => {
        $subject.update(*$subject * $change);
    };
    ( $subject:expr => / $change:expr ) => {
        $subject.update(*$subject / $change);
    };
    ( $subject:expr => $new_datum:expr ) => {
        $subject.update($new_datum);
    };
}
