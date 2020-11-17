#[derive(Copy, Clone, Debug)]
pub struct Style {
    pub top: &'static str,
    pub entry_sep: &'static str,
    pub end: &'static str,

    pub link: &'static str,

    pub title: &'static str,
    pub continuation: &'static str,

    pub stats: &'static str,
}

impl Style {
    pub const NONE: Self = Self {
        top: "",
        entry_sep: "",
        end: "",

        link: "",

        title: "",
        continuation: "",

        stats: "",
    };

    pub const BOX: Self = Self {
        top: "┌── ",
        entry_sep: "│",
        end: "└ ",

        link: "├ ",

        title: "├ ",
        continuation: "│ ",

        stats: "├ ",
    };

    pub const FANCY: Self = Self {
        top: "╭╍╍ ",
        entry_sep: "╎",
        end: "╰╍ ",

        link: "╞═ ",

        title: "├┄ ",
        continuation: "┝ ",

        stats: "├┄ ",
    };
}
