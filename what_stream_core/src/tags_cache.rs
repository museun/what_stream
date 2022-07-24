use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct TagCache {
    #[serde(skip)]
    pub path: PathBuf,
    pub cache: HashMap<Box<str>, Box<str>>,
}

impl TagCache {
    pub fn load_cache(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            ..std::fs::read(path)
                .ok()
                .and_then(|s| serde_json::from_slice::<Self>(&s).ok())
                .unwrap_or_default()
        }
    }

    pub fn sync(&self) -> anyhow::Result<()> {
        let mut writer = std::fs::File::create(&self.path).map(std::io::BufWriter::new)?;
        serde_json::to_writer(&mut writer, self)?;
        Ok(())
    }
}
