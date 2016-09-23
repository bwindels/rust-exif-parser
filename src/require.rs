#[macro_export]
macro_rules! require {
    ($opt:expr, $default:expr) => {
        match $opt {
            Some(val) => val,
            None => {
                return $default;
            }
        }
    }
}
