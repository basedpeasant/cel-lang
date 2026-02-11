
#[macro_export]
macro_rules! const_assert {
    ($e: expr) => {
        const _: () = assert!($e);
    };
    ($e: expr, $e1: expr) => {
        const _: () = assert!($e, $e1);
    };
}
