use std::time::Duration;

use anyhow::anyhow;

pub fn to_duration(text: &str) -> anyhow::Result<Duration> {
    if let Some((secs, sub_sec)) = text.trim().rsplit_once('.') {
        let sub_sec_parsed = Duration::from_secs_f64(
            sub_sec.parse::<f64>()? / 10f64.powi(sub_sec.len().try_into()?),
        );
        if let Some((mins, secs)) = secs.rsplit_once(':') {
            if let Some((hours, mins)) = mins.rsplit_once(':') {
                Ok(Duration::from_hours(hours.parse()?)
                    + Duration::from_mins(mins.parse()?)
                    + Duration::from_secs(secs.parse()?)
                    + sub_sec_parsed)
            } else {
                Ok(Duration::from_mins(mins.parse()?)
                    + Duration::from_secs(secs.parse()?)
                    + sub_sec_parsed)
            }
        } else {
            Ok(sub_sec_parsed + Duration::from_secs(secs.parse()?))
        }
    } else {
        Err(anyhow!("{text} has no period"))
    }
}

pub fn to_text(duration: Duration) -> String {
    let integer_seconds = duration.as_secs();
    let hours = integer_seconds / 3600;
    let min = integer_seconds / 60 - 60 * hours;
    let secs = integer_seconds % 60;
    let sub_secs = duration.as_millis() % 1000;
    format!("{hours}:{min:02?}:{secs:02?}.{sub_secs:03?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_round_trips() -> anyhow::Result<()> {
        let durations = [
            Duration::from_millis(140),
            Duration::from_mins(3) + Duration::from_secs(40),
            Duration::ZERO,
            Duration::from_secs_f64(314.1315),
            Duration::from_secs_f64(4523.4324),
        ];

        for d in durations {
            assert!(
                d.abs_diff(to_duration(&to_text(d))?) < Duration::from_millis(1),
                "{}",
                to_text(d)
            );
        }
        Ok(())
    }

    #[test]
    fn rand_duration_round_trips() -> anyhow::Result<()> {
        for _ in 0..100 {
            let duration = Duration::from_millis(fastrand::u64(0..2 * 3_600_000));
            assert!(duration.abs_diff(to_duration(&to_text(duration))?) < Duration::from_millis(1));
        }
        Ok(())
    }
}
