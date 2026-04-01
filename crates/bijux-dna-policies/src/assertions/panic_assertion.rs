#[macro_export]
macro_rules! policy_panic {
    ($($arg:tt)+) => {
        panic!("{}", $crate::policy_diagnostics::message(format!($($arg)+)))
    };
}
