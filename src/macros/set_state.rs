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
