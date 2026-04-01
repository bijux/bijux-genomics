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

#[macro_export]
macro_rules! policy_assert_eq {
    ($left:expr, $right:expr $(,)?) => {{
        if $left != $right {
            panic!(
                "{}\nLEFT: {:?}\nRIGHT: {:?}",
                $crate::policy_diagnostics::message(format!(
                    "{} == {}",
                    stringify!($left),
                    stringify!($right)
                )),
                $left,
                $right
            );
        }
    }};
    ($left:expr, $right:expr, $($arg:tt)+) => {{
        if $left != $right {
            panic!(
                "{}\nLEFT: {:?}\nRIGHT: {:?}",
                $crate::policy_diagnostics::message(format!($($arg)+)),
                $left,
                $right
            );
        }
    }};
}

#[macro_export]
macro_rules! policy_assert_ne {
    ($left:expr, $right:expr $(,)?) => {{
        if $left == $right {
            panic!(
                "{}\nVALUE: {:?}",
                $crate::policy_diagnostics::message(format!(
                    "{} != {}",
                    stringify!($left),
                    stringify!($right)
                )),
                $left
            );
        }
    }};
    ($left:expr, $right:expr, $($arg:tt)+) => {{
        if $left == $right {
            panic!(
                "{}\nVALUE: {:?}",
                $crate::policy_diagnostics::message(format!($($arg)+)),
                $left
            );
        }
    }};
}

#[macro_export]
macro_rules! policy_panic {
    ($($arg:tt)+) => {
        panic!("{}", $crate::policy_diagnostics::message(format!($($arg)+)))
    };
}
