//! This library generates a duration iterator for [retry](/retry/) crates.

use std::time::Duration;

use derive_builder::Builder;
use fastrand::Rng;

#[doc(hidden)]
#[derive(Debug, Builder)]
pub struct Strategy {
    /// Set initial duration.
    ///
    /// Default is 2 seconds.
    #[builder(setter(into), default = "Duration::from_secs(2)")]
    duration: Duration,

    /// Set max durations.
    ///
    /// Default is no max duration limits.
    #[builder(setter(into), default)]
    duration_max: Option<Duration>,

    #[doc(hidden)]
    #[builder(field(private), default)]
    kind: Kind,

    /// Set a duration jitter ratio.
    ///
    /// Default is `0.1`.
    #[builder(default = "0.1")]
    jitter: f32,

    #[doc(hidden)]
    #[builder(field(private), default)]
    rng: Rng,
}

/// Create a new Strategy builder.
///
/// A built iterator has infinite items, so you may want to `take()` for finite retry count.
///
/// # Examples
///
/// ```rust
/// let xs = retry_durations::builder()
///     .duration(std::time::Duration::from_secs(3))
///     .build()
///     .unwrap()
///     .take(10);
/// for x in xs {
///     println!("{x:?}");
/// }
/// ```
pub fn builder() -> StrategyBuilder {
    StrategyBuilder::default()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Fixed,
    #[default]
    Exponential,
}

impl StrategyBuilder {
    /// Select fixed interval strategy.
    pub fn fixed(&mut self) -> &mut Self {
        self.kind = Some(Kind::Fixed);
        self
    }

    /// Select exponential interval strategy. This is default.
    pub fn exponential(&mut self) -> &mut Self {
        self.kind = Some(Kind::Exponential);
        self
    }
}

impl Kind {
    pub fn next(&self, durration: Duration) -> Duration {
        match self {
            Kind::Fixed => durration,
            Kind::Exponential => durration.saturating_mul(2),
        }
    }
}

impl Strategy {
    fn j(&mut self, d: Duration) -> Duration {
        let j = (d.as_secs_f32() * self.jitter * 1000.0) as i32;
        let j = self.rng.i32((-j)..(j + 1));
        if 0 <= j {
            d.saturating_add(Duration::from_millis(j as u64))
        } else {
            d.saturating_sub(Duration::from_millis((-j) as u64))
        }
    }

    fn update_duration(&mut self) -> Duration {
        let duration = self.duration;
        let next_duration = self.kind.next(duration);

        if let Some(saturation) = self.duration_max {
            self.duration = next_duration.min(saturation);
            self.j(duration).min(saturation)
        } else {
            self.duration = next_duration;
            self.j(duration)
        }
    }
}

impl Iterator for Strategy {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.update_duration())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let xs = builder().duration(Duration::from_secs(1)).build().unwrap();
        let mut p = Duration::new(0, 0);
        for x in xs.into_iter().take(10) {
            assert!(p <= x);
            p = x;
        }

        println!("fixed");
        for x in builder().fixed().build().unwrap().take(10) {
            println!("{x:?}");
        }
        println!("exp");
        for x in builder()
            .exponential()
            .duration_max(Duration::from_secs(120))
            .build()
            .unwrap()
            .take(10)
        {
            println!("{x:?}");
        }
    }
}
