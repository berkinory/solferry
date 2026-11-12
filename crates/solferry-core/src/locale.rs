use std::iter;

/// Uses the closest language Solferry has for a tag such as `tr-TR`, `en_US` or `de_DE.UTF-8`,
/// falling back to English. The locale is process-wide, so this covers every crate's strings.
pub fn set_locale(tag: &str) {
    let tag = tag.split(['.', '@']).next().unwrap_or_default().replace('_', "-");
    let available = rust_i18n::available_locales!();
    let chosen = iter::successors(Some(tag.as_str()), |tag| tag.rfind('-').map(|end| &tag[..end]))
        .find_map(|wanted| available.iter().find(|locale| locale.eq_ignore_ascii_case(wanted)));
    rust_i18n::set_locale(chosen.map_or("en", |locale| locale));
}
