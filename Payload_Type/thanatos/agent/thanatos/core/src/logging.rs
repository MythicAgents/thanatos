#[macro_export]
macro_rules! log {
    ($($val:expr),+) => {
        #[cfg(debug_assertions)]
        {
            eprintln!(
                "[{}:{}:{}] {}",
                std::file!(),
                std::line!(),
                std::column!(),
                format_args!($($val),+),
            );
        }
    };
}
