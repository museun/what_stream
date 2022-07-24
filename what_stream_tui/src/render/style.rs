use std::borrow::Cow;

// TODO these names probably shouldn't be displayed to the user
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Style {
    pub top: Cow<'static, str>,
    pub entry_sep: Cow<'static, str>,
    pub end: Cow<'static, str>,

    pub link: Cow<'static, str>,

    pub title: Cow<'static, str>,
    pub continuation: Cow<'static, str>,

    pub stats: Cow<'static, str>,
    // TODO add 'tags' section
}

impl Default for Style {
    fn default() -> Self {
        Self::BOX
    }
}

const fn s(s: &str) -> Cow<'_, str> {
    Cow::Borrowed(s)
}

impl Style {
    pub const NONE: Self = Self {
        top: s(""),
        entry_sep: s(""),
        end: s(""),

        link: s(""),

        title: s(""),
        continuation: s(""),

        stats: s(""),
    };

    pub const BOX: Self = Self {
        top: s("┌── "),
        entry_sep: s("│"),
        end: s("└ "),

        link: s("├ "),

        title: s("├ "),
        continuation: s("│ "),

        stats: s("├ "),
    };
}
