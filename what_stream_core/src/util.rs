pub fn format_time(started_at: &str) -> (i64, String) {
    use chrono::*;
    let duration: Duration = Utc::now()
        - started_at
            .parse::<DateTime<Utc>>()
            .expect("valid timestamp");

    let seconds = duration.num_seconds();
    let hours = (seconds / 60) / 60;
    let minutes = (seconds / 60) % 60;

    let started = if hours > 0 {
        format!(
            "{hours} hour{h_plural} {minutes} minute{m_plural}",
            hours = hours,
            minutes = minutes,
            h_plural = if hours > 1 { "s" } else { "" },
            m_plural = if minutes > 1 { "s" } else { "" },
        )
    } else {
        format!(
            "{minutes} minute{m_plural}",
            minutes = minutes,
            m_plural = if minutes > 1 { "s" } else { "" }
        )
    };

    (seconds, started)
}

pub fn trim_word_boundaries(s: &str) -> &str {
    const HEAD: &str = "([{";
    const TAIL: &str = ",.!?-:}])";
    s.trim_start_matches(|c| HEAD.contains(c))
        .trim_end_matches(|c| TAIL.contains(c))
        .trim()
}
