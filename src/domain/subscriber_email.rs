use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Into<String> for SubscriberEmail {
    fn into(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use proptest::prelude::*;
    use proptest::strategy::{NewTree, ValueTree};
    use proptest::test_runner::TestRunner;

    struct EmailGeneratorValueTree {
        email: String,
    }

    impl ValueTree for EmailGeneratorValueTree {
        type Value = String;
        fn current(&self) -> Self::Value {
            self.email.clone()
        }
        fn simplify(&mut self) -> bool {
            false
        }

        fn complicate(&mut self) -> bool {
            false
        }
    }

    #[derive(Debug)]
    struct EmailGenerator;
    impl Strategy for EmailGenerator {
        type Tree = EmailGeneratorValueTree;
        type Value = String;
        fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
            Ok(EmailGeneratorValueTree {
                email: SafeEmail().fake_with_rng(runner.rng()),
            })
        }
    }

    proptest! {
      #[test]
      fn valid_emails_are_parsed_successfully(email in EmailGenerator) {
        prop_assert!(SubscriberEmail::parse(email).is_ok());
      }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
