use image::{DynamicImage, ImageError};
use log::info;
use tokio::fs;

#[derive(Clone)]
pub struct FileService {
    pub storage_path: String,
}

impl FileService {
    pub async fn save_image(
        &self,
        folder: &str,
        filename: &str,
        filetype: &str,
        image: &DynamicImage,
    ) -> Result<String, ImageError> {
        let full_folder = format!("{}/{}", self.storage_path, folder);
        fs::create_dir_all(&full_folder).await?;
        let full_path = format!("{}/{}.{}", full_folder, filename, filetype);
        image.save(&full_path)?;
        info!("Saved image: {}", &full_path);
        Ok(format!("{}/{}.{}", folder, filename, filetype))
    }
}
