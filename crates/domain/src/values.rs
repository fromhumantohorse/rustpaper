use crate::shared::bounded_value;

bounded_value!(Volume, u8, 0, 100, 80);
bounded_value!(PlaybackSpeed, f32, 0.25, 4.0, 1.0);
bounded_value!(ColorLevel, i8, -100, 100, 0);
bounded_value!(FpsLimit, u16, 10, 240, 60);
bounded_value!(RotationMinutes, u16, 1, 1440, 15);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounded, DomainError};

    #[test]
    fn accepts_values_inside_bounds() {
        assert_eq!(Volume::try_new(100).map(Bounded::get), Ok(100));
    }

    #[test]
    fn rejects_values_outside_bounds() {
        assert!(matches!(
            PlaybackSpeed::try_new(8.0),
            Err(DomainError::OutOfRange {
                field: "PlaybackSpeed",
                ..
            })
        ));
    }

    #[test]
    fn rejects_nan() {
        assert!(PlaybackSpeed::try_new(f32::NAN).is_err());
    }

    #[test]
    fn deserialization_enforces_bounds() {
        assert!(serde_json::from_str::<FpsLimit>("5").is_err());
        assert_eq!(
            serde_json::from_str::<FpsLimit>("30")
                .map(Bounded::get)
                .ok(),
            Some(30)
        );
    }
}
