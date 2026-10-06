use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    pub fn parse(text: &str) -> Option<Self> {
        let core = text
            .trim()
            .trim_start_matches('v')
            .split(['-', '+'])
            .next()?;
        let mut parts = core.split('.').map(str::parse::<u32>);
        let version = Self {
            major: parts.next()?.ok()?,
            minor: parts.next()?.ok()?,
            patch: parts.next()?.ok()?,
        };
        parts.next().is_none().then_some(version)
    }

    pub fn current() -> Self {
        Self::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Self {
            major: 0,
            minor: 0,
            patch: 0,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags_and_orders_numerically() {
        let older = Version::parse("v0.9.0").unwrap();
        let newer = Version::parse("0.10.0").unwrap();
        assert!(newer > older);
        assert_eq!(newer.to_string(), "0.10.0");
    }

    #[test]
    fn ignores_pre_release_suffixes_and_rejects_garbage() {
        assert_eq!(Version::parse("v1.2.3-beta.1"), Version::parse("1.2.3"));
        assert!(Version::parse("1.2").is_none());
        assert!(Version::parse("1.2.3.4").is_none());
        assert!(Version::parse("latest").is_none());
    }
}
