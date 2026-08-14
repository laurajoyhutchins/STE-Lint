#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_lexicon_resolves_only_explicit_forms() {
        let lexicon = RuntimeLexicon::embedded().unwrap();

        let ensure = lexicon.lookup_form("ensures").unwrap();
        assert_eq!(ensure.lemma, "ensure");
        assert_eq!(ensure.status, ApprovalStatus::Unapproved);

        let permitted = lexicon.lookup_form("permitted").unwrap();
        assert_eq!(permitted.lemma, "PERMITTED");
        assert_eq!(permitted.status, ApprovalStatus::Approved);

        assert!(lexicon.lookup_form("permitting").is_none());
    }
}
