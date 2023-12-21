use std::borrow::Cow;

use teloxide::{
    dispatching::DpHandlerDescription,
    dptree::{di::DependencyMap, Handler},
};

pub type ReturnType<Output> = Handler<'static, DependencyMap, Output, DpHandlerDescription>;

pub fn create_username_or_default<'opt>(
    default: &'opt str,
    username: Option<&'opt String>,
) -> Cow<'opt, str> {
    username.map_or_else(
        || Cow::Borrowed(default),
        |username| Cow::Owned(format!("@{}", username)),
    )
}

pub trait FormatArgument {
    fn end_with_comma_if_not_empty(&self) -> Cow<str>;
}

impl FormatArgument for String {
    fn end_with_comma_if_not_empty(&self) -> Cow<str> {
        if !self.is_empty() {
            Cow::Owned(format!("{}, ", self))
        } else {
            Cow::Borrowed(self)
        }
    }
}
