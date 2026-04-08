pub fn get_optional_flag(matches: Option<&clap::ArgMatches>, id: &str) -> bool {
    matches.filter(|m| m.try_contains_id(id).unwrap_or(false)).map(|m| m.get_flag(id)).unwrap_or(false)
}

pub fn get_optional_value<'a, T: Clone + Send + Sync + 'static>(
    matches: Option<&'a clap::ArgMatches>,
    id: &str,
) -> Option<&'a T> {
    matches.filter(|m| m.try_contains_id(id).unwrap_or(false)).and_then(|m| m.try_get_one::<T>(id).ok().flatten())
}
