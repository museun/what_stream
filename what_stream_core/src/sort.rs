use anyhow::Context as _;

#[derive(Debug, Copy, Clone)]
pub struct SortAction {
    pub column: Column,
    pub direction: Direction,
}

impl std::str::FromStr for SortAction {
    type Err = anyhow::Error;
    fn from_str(flag: &str) -> anyhow::Result<Self> {
        let mut iter = flag.splitn(2, ',');
        let head = iter.next().with_context(|| "a column must be provided")?;
        let column = match head {
            "viewers" => Column::Viewers,
            "uptime" => Column::Uptime,
            "name" => Column::Name,
            name => anyhow::bail!(
                "invalid column: {}. supported columns: [viewers | uptime | name]",
                name
            ),
        };

        let direction = iter
            .next()
            .map(|tail| match tail {
                "asc" | "ascending" => Ok(Direction::Ascending),
                "desc" | "descending" | "" => Ok(Direction::Descending),
                dir => anyhow::bail!("invalid direction: {}. supported directions: [asc | ascending | desc | descending]", dir),
            })
            .transpose()?
            .unwrap_or(Direction::Descending);

        Ok(Self { column, direction })
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Column {
    Viewers,
    Uptime,
    Name,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum Direction {
    Descending,
    Ascending,
}
