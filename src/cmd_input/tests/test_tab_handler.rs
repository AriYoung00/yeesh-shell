#[cfg(test)]
mod tab_handler_tests {
    use filesystem::FakeFileSystem;

    use crate::cmd_input::suggester::SuggestionType::File;
    use crate::cmd_input::suggester::{Suggester, Suggestion, SuggestionType};
    use crate::cmd_input::TabHandler;

    #[derive(Clone)]
    struct TestSuggester {
        suggestions:        Vec<Suggestion>,
        get_suggestion_cnt: usize,
    }

    impl From<(&'static str, bool, SuggestionType)> for Suggestion {
        fn from((replacement, is_prefix, s_type): (&'static str, bool, SuggestionType)) -> Self {
            Suggestion {
                replacement: replacement.to_string(),
                is_prefix,
                s_type,
            }
        }
    }

    impl TestSuggester {
        pub fn new(suggestions: Vec<(&'static str, bool, SuggestionType)>) -> Self {
            TestSuggester {
                suggestions:        suggestions.into_iter().map(|x| x.into()).collect(),
                get_suggestion_cnt: 0,
            }
        }
    }

    impl Suggester for TestSuggester {
        fn get_suggestions(&mut self, _: &str) -> Vec<Suggestion> {
            self.get_suggestion_cnt += 1;
            self.suggestions.to_vec()
        }

        fn get_get_suggestion_count(&self) -> usize {
            self.get_suggestion_cnt
        }
    }

    fn setup(suggesters: Vec<Box<dyn Suggester>>) -> TabHandler {
        let mut handler = TabHandler::new(FakeFileSystem::new());
        handler.set_suggesters(suggesters);
        handler
    }

    #[test]
    fn test_suggestion_cache() {
        let suggestions = vec![("hello", true, File), ("there", false, File)];
        let suggester = TestSuggester::new(suggestions.clone());
        let mut handler = setup(vec![Box::new(suggester)]);

        let a = &"a".to_string();
        let b = &"b".to_string();

        handler.get_suggestion(a);
        let suggest_count_save = handler.get_suggesters()[0].get_get_suggestion_count();
        handler.get_suggestion(a);
        assert_eq!(
            suggest_count_save,
            handler.get_suggesters()[0].get_get_suggestion_count()
        );

        handler.refresh();
        handler.get_suggestion(b);
        assert_eq!(
            suggest_count_save + 1,
            handler.get_suggesters()[0].get_get_suggestion_count()
        )
    }

    #[test]
    fn test_suggestions_loop() {
        let suggestions = vec![("hello", true, File), ("there", false, File)];
        let suggester = TestSuggester::new(suggestions.clone());
        let mut handler = setup(vec![Box::new(suggester)]);

        let a = &"".to_string();

        let s = handler.get_suggestion(a);
        assert!(s.is_some());
        assert_eq!(s.unwrap(), suggestions[0].0);

        let s = handler.get_suggestion(a);
        assert!(s.is_some());
        assert_eq!(s.unwrap(), suggestions[1].0);

        let s = handler.get_suggestion(a);
        assert!(s.is_some());
        assert_eq!(s.unwrap(), suggestions[0].0);
    }
}
