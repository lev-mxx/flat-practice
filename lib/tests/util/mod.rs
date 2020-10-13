
#[macro_export]
macro_rules! text {
    ($name: expr) => { paste::paste! { from_utf8(include_bytes!(concat!("data/", $name))) } };
}
