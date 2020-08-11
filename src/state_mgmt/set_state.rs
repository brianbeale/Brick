#[macro_export]
macro_rules! set {
    ( $subject:expr => + $change:expr ) => {
        let val = $subject.borrow().read();
        $subject.borrow_mut().update(val + $change);
    };
    ( $subject:expr => - $change:expr ) => {
        let val = $subject.borrow().read();
        $subject.borrow_mut().update(val - $change);
    };
    ( $subject:expr => * $change:expr ) => {
        let val = $subject.borrow().read();
        $subject.borrow_mut().update(val * $change);
    };
    ( $subject:expr => / $change:expr ) => {
        let val = $subject.borrow().read();
        $subject.borrow_mut().update(val / $change);
    };
    ( $subject:expr => $new_datum:expr ) => {
        $subject.borrow_mut().update($new_datum);
    };
}
