use chrono::Local;

pub(crate) fn generate_header(template: &str) -> String {
    generate_header_with_time(template, Local::now())
}

pub(crate) fn render_datetime_template(template: &str) -> String {
    render_datetime_template_with_time(template, Local::now())
}

pub(crate) fn render_datetime_template_with_time<Tz: chrono::TimeZone>(
    template: &str,
    dt: chrono::DateTime<Tz>,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    // 12h-Zeitwerte vorbereiten
    let hour_12 = dt.format("%I").to_string();
    let hour_12_nopad = if hour_12.starts_with('0') {
        hour_12[1..].to_string()
    } else {
        hour_12.clone()
    };
    let ampm_lower = dt.format("%P").to_string();
    let ampm_upper = dt.format("%p").to_string();

    // Token -> Wert Zuordnung.
    // Laengere Token zuerst, damit z.B. "hh" vor "h" matched.
    let tokens: &[(&str, String)] = &[
        ("YYYY", dt.format("%Y").to_string()),
        ("MM", dt.format("%m").to_string()),
        ("DD", dt.format("%d").to_string()),
        ("HH", dt.format("%H").to_string()),
        ("hh", hour_12),
        ("mm", dt.format("%M").to_string()),
        ("ss", dt.format("%S").to_string()),
        ("h", hour_12_nopad),
        ("a", ampm_lower),
        ("A", ampm_upper),
    ];

    // Single-Pass: jedes Token wird nur im Original-Template erkannt,
    // sodass eingesetzte Werte (z.B. "pm" enthaelt 'a') nie erneut ersetzt werden.
    let mut result = String::with_capacity(template.len() * 2);
    let mut remaining = template;
    while !remaining.is_empty() {
        if let Some((tok, val)) = tokens.iter().find(|(tok, _)| remaining.starts_with(tok)) {
            result.push_str(val);
            remaining = &remaining[tok.len()..];
        } else {
            // Safety: wir nehmen ein UTF-8-Zeichen, nicht nur ein Byte.
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }

    result
}

pub(crate) fn generate_header_with_time<Tz: chrono::TimeZone>(
    template: &str,
    dt: chrono::DateTime<Tz>,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    render_datetime_template_with_time(template, dt)
}
