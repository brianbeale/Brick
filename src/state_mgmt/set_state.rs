#[macro_export]
macro_rules! set {
    ( $subject:expr => + $change:expr ) => {{
        let __v = $subject.read();
        $subject.set(__v + $change);
    }};
    ( $subject:expr => - $change:expr ) => {{
        let __v = $subject.read();
        $subject.set(__v - $change);
    }};
    ( $subject:expr => * $change:expr ) => {{
        let __v = $subject.read();
        $subject.set(__v * $change);
    }};
    ( $subject:expr => / $change:expr ) => {{
        let __v = $subject.read();
        $subject.set(__v / $change);
    }};
    ( $subject:expr => $new_datum:expr ) => {
        $subject.set($new_datum);
    };
}
