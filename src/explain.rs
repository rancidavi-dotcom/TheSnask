include!(concat!(env!("OUT_DIR"), "/explain.rs"));

#[cfg(test)]
mod tests {
    use super::get_explanation;

    #[test]
    fn explain_has_entry_for_missing_paren() {
        let text = get_explanation("S1002").expect("S1002 should be documented");
        assert!(text.contains("missing closing `)`"));
        assert!(text.contains("severity"));
    }

    #[test]
    fn explain_has_entry_for_unknown_variable() {
        let text = get_explanation("S2002").expect("S2002 should be documented");
        assert!(text.contains("variable"));
        assert!(text.contains("severity"));
    }

    #[test]
    fn explain_returns_none_for_unknown_code() {
        assert!(get_explanation("S9999").is_none());
    }
}
