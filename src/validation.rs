/// A single validation rule for a value of type `T`.
///
/// Any `Fn(&T) -> Result<(), String>` automatically implements this trait, so
/// plain functions and closures work as validators without any wrapper:
///
/// ```rust,ignore
/// fn no_spaces(s: &String) -> Result<(), String> {
///     if s.contains(' ') { Err("No spaces allowed".into()) } else { Ok(()) }
/// }
///
/// #[model]
/// struct LoginForm {
///     #[validate(required, no_spaces)]
///     username: String,
/// }
/// ```
pub trait Validate<T> {
    fn validate(&self, value: &T) -> Result<(), String>;
}

impl<T, F: Fn(&T) -> Result<(), String>> Validate<T> for F {
    fn validate(&self, value: &T) -> Result<(), String> {
        self(value)
    }
}

/// Implemented by types that have a meaningful notion of "empty". Used by the
/// `required` built-in validator. Implement this on custom types to support
/// `#[validate(required)]`.
pub trait IsPresent {
    fn is_present(&self) -> bool;
}
impl IsPresent for String {
    fn is_present(&self) -> bool {
        !self.trim().is_empty()
    }
}
impl<T> IsPresent for Option<T> {
    fn is_present(&self) -> bool {
        self.is_some()
    }
}

/// Implemented by numeric types to support the `positive` and `non_negative`
/// built-in validators. Automatically implemented for all primitive integer and
/// float types; implement on custom numeric wrappers if needed.
pub trait IsPositive {
    fn is_positive(&self) -> bool;
    fn is_non_negative(&self) -> bool;
}
macro_rules! impl_is_positive_signed {
    ($($t:ty),*) => {
        $(impl IsPositive for $t {
            fn is_positive(&self) -> bool { *self > 0 as $t }
            fn is_non_negative(&self) -> bool { *self >= 0 as $t }
        })*
    };
}
macro_rules! impl_is_positive_unsigned {
    ($($t:ty),*) => {
        $(impl IsPositive for $t {
            fn is_positive(&self) -> bool { *self > 0 }
            fn is_non_negative(&self) -> bool { true }
        })*
    };
}
impl_is_positive_signed!(i8, i16, i32, i64, i128, isize, f32, f64);
impl_is_positive_unsigned!(u8, u16, u32, u64, u128, usize);

/// Implemented by string-like types to support the `alpha` and `alphanumeric`
/// built-in validators. Implement on custom string wrappers if needed.
pub trait IsAlpha {
    fn is_alpha(&self) -> bool;
    fn is_alphanumeric(&self) -> bool;
}
impl IsAlpha for String {
    fn is_alpha(&self) -> bool {
        self.chars().all(|c| c.is_alphabetic())
    }
    fn is_alphanumeric(&self) -> bool {
        self.chars().all(|c| c.is_alphanumeric())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Validate blanket impl ─────────────────────────────────────────────────

    #[test]
    fn fn_is_a_validator() {
        fn no_x(s: &String) -> Result<(), String> {
            if s.contains('x') {
                Err("no x".into())
            } else {
                Ok(())
            }
        }
        assert!(no_x.validate(&"hello".to_string()).is_ok());
        assert!(no_x.validate(&"hex".to_string()).is_err());
    }

    #[test]
    fn closure_is_a_validator() {
        let min3 = |s: &String| -> Result<(), String> {
            if s.len() >= 3 {
                Ok(())
            } else {
                Err("too short".into())
            }
        };
        assert!(min3.validate(&"abc".to_string()).is_ok());
        assert!(min3.validate(&"ab".to_string()).is_err());
    }

    // ── IsPresent ─────────────────────────────────────────────────────────────

    #[test]
    fn is_present_non_empty_string() {
        assert!("hello".to_string().is_present());
    }

    #[test]
    fn is_present_empty_string() {
        assert!(!String::new().is_present());
    }

    #[test]
    fn is_present_whitespace_string() {
        assert!(!"  ".to_string().is_present());
    }

    #[test]
    fn is_present_some_option() {
        assert!(Some(42).is_present());
    }

    #[test]
    fn is_present_none_option() {
        assert!(!Option::<i32>::None.is_present());
    }

    // ── IsPositive ────────────────────────────────────────────────────────────

    #[test]
    fn is_positive_signed_positive() {
        assert!(1i32.is_positive());
        assert!(!0i32.is_positive());
        assert!(!(-1i32).is_positive());
    }

    #[test]
    fn is_non_negative_signed() {
        assert!(0i32.is_non_negative());
        assert!(1i32.is_non_negative());
        assert!(!(-1i32).is_non_negative());
    }

    #[test]
    fn is_positive_unsigned() {
        assert!(1u32.is_positive());
        assert!(!0u32.is_positive());
    }

    #[test]
    fn is_non_negative_unsigned_always_true() {
        assert!(0u32.is_non_negative());
        assert!(u32::MAX.is_non_negative());
    }

    #[test]
    fn is_positive_float() {
        // Use UFCS to call our IsPositive trait method, not f64's deprecated primitive method.
        assert!(IsPositive::is_positive(&0.1f64));
        assert!(!IsPositive::is_positive(&0.0f64));
        assert!(!IsPositive::is_positive(&-0.1f64));
    }

    // ── IsAlpha ───────────────────────────────────────────────────────────────

    #[test]
    fn is_alpha_letters_only() {
        assert!("Hello".to_string().is_alpha());
    }

    #[test]
    fn is_alpha_rejects_digits() {
        assert!(!"Hello1".to_string().is_alpha());
    }

    #[test]
    fn is_alphanumeric_accepts_letters_and_digits() {
        assert!("Hello123".to_string().is_alphanumeric());
    }

    #[test]
    fn is_alphanumeric_rejects_symbols() {
        assert!(!"Hello!".to_string().is_alphanumeric());
    }

    #[test]
    fn is_alpha_empty_string() {
        assert!("".to_string().is_alpha());
    }
}
