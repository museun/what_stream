pub enum LinePartition<'a> {
    Start(&'a str),
    Continuation(&'a str),
}

pub fn partition_line(
    input: &str,
    max: usize,
    left: usize,
) -> impl Iterator<Item = LinePartition<'_>> + '_ {
    use {
        unicode_segmentation::UnicodeSegmentation as _, //
        unicode_width::UnicodeWidthStr as _,
    };
    let mut budget = max;
    input.split_word_bounds().map(move |word| {
        let word = word.trim_end_matches('\n');
        let width = word.width();
        match budget.checked_sub(width) {
            Some(n) => {
                budget = n;
                LinePartition::Continuation(word)
            }
            None => {
                budget = max - width - left;
                LinePartition::Start(word)
            }
        }
    })
}
