use crate::public::constant::storage::get_data_path;
use crate::public::structure::config::APP_CONFIG;
use log::info;

pub fn initialize_folder() {
    let (data_home, image_home, upload_folder) = {
        let config = APP_CONFIG.get().unwrap().read().unwrap();
        let data_home = config
            .data_home
            .clone()
            .unwrap_or_else(|| get_data_path().clone());
        let image_home = config.image_home.clone();
        let upload_folder = config.upload_folder.clone();
        (data_home, image_home, upload_folder)
    };

    info!("Storage root initialized at: {}", data_home.display());
    std::fs::create_dir_all(data_home.join("db")).unwrap();
    std::fs::create_dir_all(data_home.join("object/compressed")).unwrap();

    // Pre-create image root and uploads directory from config
    if let Some(ref root) = image_home {
        info!("Creating image root: {}", root.display());
        std::fs::create_dir_all(root).unwrap();
        std::fs::create_dir_all(root.join(&upload_folder)).unwrap();
    }
}
