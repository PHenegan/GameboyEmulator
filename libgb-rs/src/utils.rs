
pub trait Merge<T, R> {
    /// Combine this element with another type to create a single result
    fn merge(self, b: T) -> R;
}

pub trait Split<T> {
    /// Divide this element into two smaller elements of another type
    fn split(self) -> (T, T);
}

impl Merge<u8, u16> for u8 {
    fn merge(self, b: u8) -> u16 {
        return (b as u16) + ((self as u16) << 8);
    }
}

impl Split<u8> for u16 {
    fn split(self) -> (u8, u8) {
        let left = (self >> 8) as u8;
        let right = self as u8;

        (left, right)
    }
}
