pub mod config;
pub mod error;
pub mod preprocessor {
    use mdbook::{preprocess::{PreprocessorContext, Preprocessor}, book::Book, errors::Error};

    /// A no-op preprocessor.
    pub struct RFCPreprocessor;

    impl RFCPreprocessor {
        pub fn new() -> RFCPreprocessor {
            RFCPreprocessor
        }
    }

    impl Preprocessor for RFCPreprocessor {
        fn name(&self) -> &str {
            "rfc-preprocessor"
        }

        fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
            // In testing we want to tell the preprocessor to blow up by setting a
            // particular config value
            if let Some(rfc_cfg) = ctx.config.get_preprocessor(self.name()) {
                if rfc_cfg.contains_key("blow-up") {
                    //anyhow::bail!("Boom!!1!");
                }
            }

            // we *are* a no-op preprocessor after all
            Ok(book)
        }

        fn supports_renderer(&self, renderer: &str) -> bool {
            ["rfc"].contains(&renderer)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn rfc_preprocessor_run() {
            let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "rfc": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
            let input_json = input_json.as_bytes();

            let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
            let expected_book = book.clone();
            let result = RFCPreprocessor::new().run(&ctx, book);
            assert!(result.is_ok());

            // The rfc-preprocessor should not have made any changes to the book content.
            let actual_book = result.unwrap();
            assert_eq!(actual_book, expected_book);
        }
    }
}