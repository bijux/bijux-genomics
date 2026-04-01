#[macro_export]
macro_rules! policy_assert {
    ($cond:expr $(,)?) => {
        assert!(
            $cond,
            "{}",
            $crate::policy_diagnostics::message(stringify!($cond))
        );
    };
    ($cond:expr, $($arg:tt)+) => {
        assert!(
            $cond,
            "{}",
            $crate::policy_diagnostics::message(format!($($arg)+))
        );
    };
}
