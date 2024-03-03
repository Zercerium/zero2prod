use super::{SubscriberEmail, SubscriberName};

#[derive(Clone)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
