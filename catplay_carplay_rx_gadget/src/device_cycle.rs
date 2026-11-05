/// Stable ordering avoids depending on BlueZ's hash-map iteration order.
pub(crate) fn next_device<T: Copy + Eq>(devices: &[T], current: Option<T>) -> Option<T> {
    if let Some(index) = current.and_then(|current| devices.iter().position(|v| *v == current)) {
        return (devices.len() > 1).then(|| devices[(index + 1) % devices.len()]);
    }
    devices.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycle_wraps_and_skips_current() {
        assert_eq!(next_device(&[1, 2, 3], Some(2)), Some(3));
        assert_eq!(next_device(&[1, 2, 3], Some(3)), Some(1));
    }
    #[test]
    fn no_alternative_does_not_disconnect() {
        assert_eq!(next_device::<u8>(&[], Some(1)), None);
        assert_eq!(next_device(&[1], Some(1)), None);
        assert_eq!(next_device(&[2], Some(1)), Some(2));
        assert_eq!(next_device(&[1, 2], None), Some(1));
    }
}
