use adder;
mod utils;
pub fn subtract(left: i32, right: i32) -> i32 {
    adder::add(left, utils::negator::negate(right))
}