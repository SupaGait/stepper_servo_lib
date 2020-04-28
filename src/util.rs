/// Clamp the value between min an max.
#[inline]
pub fn clamp<T>(min: T, max: T, value: T) -> T
where
    T: PartialOrd,
{
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}
