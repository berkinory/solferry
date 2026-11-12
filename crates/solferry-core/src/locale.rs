use std::iter;

/// Uses the closest language Solferry has for a tag such as `tr-TR`, `en_US` or `de_DE.UTF-8`,
/// falling back to English. The locale is process-wide, so this covers every crate's strings.
pub fn set_locale(tag: &str) {
    let tag = tag.split(['.', '@']).next().unwrap_or_default().replace('_', "-");
    let tag = chinese_script(&tag).map_or(tag, str::to_owned);
    let available = rust_i18n::available_locales!();
    let chosen = iter::successors(Some(tag.as_str()), |tag| tag.rfind('-').map(|end| &tag[..end]))
        .find_map(|wanted| available.iter().find(|locale| locale.eq_ignore_ascii_case(wanted)));
    rust_i18n::set_locale(chosen.map_or("en", |locale| locale));
}

/// Chinese is kept by script, but Linux and Windows often name a region instead, as in `zh-TW`.
fn chinese_script(tag: &str) -> Option<&'static str> {
    let mut parts = tag.split('-');
    if !parts.next()?.eq_ignore_ascii_case("zh") {
        return None;
    }
    match parts.next() {
        Some(part) if part.eq_ignore_ascii_case("hans") || part.eq_ignore_ascii_case("hant") => {
            None
        }
        Some(part) if ["tw", "hk", "mo"].iter().any(|region| part.eq_ignore_ascii_case(region)) => {
            Some("zh-Hant")
        }
        _ => Some("zh-Hans"),
    }
}
