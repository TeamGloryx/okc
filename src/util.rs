pub fn check(condition: bool) -> Option<()> {
    if !condition {
        return None;
    }

    Some(())
}
