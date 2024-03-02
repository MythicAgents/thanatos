#[macro_export]
macro_rules! debug {
    () => {
        #[cfg(debug_assertions)]
        {
            eprintln!("[{}:{}:{}]", std::file!(), std::line!(), std::column!())
        }
        #[cfg(not(debug_assertions))]
        {}
    };

    ($val:expr $(,)?) => {
        #[cfg(debug_assertions)]
        {
            match $val {
                tmp => {
                    eprintln!(
                        "[{}:{}:{}] {} = {:#?}",
                        std::file!(),
                        std::line!(),
                        std::column!(),
                        stringify!($val),
                        &tmp
                    );
                    tmp
                }
            }
        }
    };

    ($($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        {
            ($($crate::debug!($val)),+,)
        }
        #[cfg(not(debug_assertions))]
        {}
    };
}
