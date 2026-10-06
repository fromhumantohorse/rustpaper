use crate::DomainError;

pub trait Bounded: Sized + Copy {
    type Raw: Copy + PartialOrd;
    const MIN: Self::Raw;
    const MAX: Self::Raw;

    fn get(self) -> Self::Raw;
    fn try_new(raw: Self::Raw) -> Result<Self, DomainError>;
}

macro_rules! bounded_value {
    ($name:ident, $raw:ty, $min:expr, $max:expr, $default:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name($raw);

        impl $crate::shared::Bounded for $name {
            type Raw = $raw;
            const MIN: $raw = $min;
            const MAX: $raw = $max;

            fn get(self) -> $raw {
                self.0
            }

            fn try_new(raw: $raw) -> Result<Self, $crate::DomainError> {
                if (Self::MIN..=Self::MAX).contains(&raw) {
                    return Ok(Self(raw));
                }
                Err($crate::DomainError::OutOfRange {
                    field: stringify!($name),
                    value: f64::from(raw),
                    min: f64::from(Self::MIN),
                    max: f64::from(Self::MAX),
                })
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = <$raw as serde::Deserialize>::deserialize(deserializer)?;
                <Self as $crate::shared::Bounded>::try_new(raw).map_err(serde::de::Error::custom)
            }
        }
    };
}

pub(crate) use bounded_value;
