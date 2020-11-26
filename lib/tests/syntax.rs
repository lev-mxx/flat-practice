use anyhow::Result;

use flat_practice_lib::syntax;
use std::str::from_utf8;

mod util;

macro_rules! test {
    ($script: expr, $expected: expr) => {
        paste::paste! {
            #[test]
            fn [<test _ $script>]() -> Result<()> {
                assert_eq!($expected, syntax::check(text!(concat!("scripts/", $script))?)?);
                Ok(())
            }
        }
    };
}

test!("empty", true);
test!("word", false);
test!("open", true);
test!("let", true);
test!("get", true);
test!("cond", true);
test!("complex", true);
