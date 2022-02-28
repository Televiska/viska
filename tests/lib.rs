macro_rules! p {
    ($variable:expr) => {
        panic!("{:?} deliberatly panicked, but it shouldn't", $variable)
    };
    ($variable1:expr, $variable2:expr) => {
        panic!(
            "{:?}({:?}) deliberatly panicked, but it shouldn't",
            $variable1, $variable2
        )
    };
}

pub mod common;
pub mod unit_tests;
