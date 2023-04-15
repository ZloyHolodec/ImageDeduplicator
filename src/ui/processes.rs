use crate::database::AcquiredConnection;
use crate::database::Database;
use crate::database::FolderWrapper;
use crate::database::ImageWrapper;
use crate::filesystem::find_file_recursive;
use gtk::glib::Sender;
use image;
use std::fs;
use std::sync::mpsc;
use std::thread;
use tokio;

const HASH_WORKERS: usize = 8;

pub enum ScanFolderStatus {
    ScanningFolders(String),
    ImageFound(String),
    HashCalculated(String),
    Done,
}

pub enum HashingStatus {
    NewHash(ImageWrapper),
    Done,
}

#[tokio::main]
pub async fn insert_new_folders(paths: Vec<String>) -> Vec<FolderWrapper> {
    let connection = Database::connect_default().await;
    let mut connection_pool = connection.get_connection().await;
    connection_pool.insert_folders(paths).await
}

#[tokio::main]
pub async fn scan_folders(sender: Sender<ScanFolderStatus>) {
    let extensions = vec![".jpg".to_string(), ".jpeg".to_string(), ".png".to_string()];
    let connection = Database::connect_default().await;
    let mut connection_pool = connection.get_connection().await;

    let folders = connection_pool.get_all_folders().await.unwrap();

    for folder in folders.iter() {
        sender
            .send(ScanFolderStatus::ScanningFolders(folder.path.clone()))
            .unwrap();

        let images = find_file_recursive(folder.path.clone(), &extensions);

        for image in images.iter() {
            let result = connection_pool.insert_image(image).await;
            match result {
                Ok(()) => {
                    sender
                        .send(ScanFolderStatus::ImageFound(image.clone()))
                        .unwrap();
                }
                Err(_) => {}
            };
        }
    }

    start_hashing(connection_pool, &sender).await;
    sender.send(ScanFolderStatus::Done).unwrap();
}

async fn start_hashing(mut connection_pool: AcquiredConnection, sender: &Sender<ScanFolderStatus>) {
    let images = connection_pool.get_non_hashed_images().await.unwrap();
    let mut images = split_images_for_processing(images, HASH_WORKERS);
    let mut receivers = Vec::new();

    for _ in 0..HASH_WORKERS {
        let images_chunk = images.pop().unwrap();
        let (sender, receiver) = mpsc::channel();

        receivers.push(receiver);
        thread::spawn(move || {
            do_hashing(images_chunk, sender);
        });
    }

    let mut done_count = 0;

    while done_count < receivers.len() - 1 {
        for receiver in receivers.iter() {
            let message = match receiver.recv() {
                Ok(val) => val,
                _ => {
                    continue;
                }
            };

            match message {
                HashingStatus::NewHash(image) => {
                    connection_pool
                        .update_image_hash(image.id, image.hash)
                        .await
                        .unwrap();
                    sender
                        .send(ScanFolderStatus::HashCalculated(image.path.clone()))
                        .unwrap();
                }
                HashingStatus::Done => {
                    done_count += 1;
                }
            }
        }
    }
}

fn do_hashing(images: Vec<ImageWrapper>, sender: mpsc::Sender<HashingStatus>) {
    for image in images {
        let hash_result = get_image_hash(&image.path);
        let mut image_to_send = image.clone();
        image_to_send.hash = hash_result;
        sender.send(HashingStatus::NewHash(image_to_send)).unwrap();
    }

    sender.send(HashingStatus::Done).unwrap();
}

fn get_image_hash(path: &String) -> Option<i64> {
    let img = image::open(path);

    return match img {
        Ok(img) => Some(calc_hash(img)),
        Err(_) => None,
    };
}

fn calc_hash(img: image::DynamicImage) -> i64 {
    let img = img.resize_exact(8, 8, image::imageops::FilterType::Nearest);
    let img = img.grayscale();

    let mut light_medium: u64 = 0;
    for pixel in img.as_bytes() {
        light_medium += *pixel as u64;
    }

    light_medium = light_medium / 64;

    let mut hash: i64 = 0;
    let mut marker: i64 = 1;
    for pixel in img.as_bytes() {
        if *pixel as u64 > light_medium {
            hash |= marker;
        }
        marker = marker << 1;
    }

    hash
}

fn split_images_for_processing(
    mut images: Vec<ImageWrapper>,
    size: usize,
) -> Vec<Vec<ImageWrapper>> {
    let mut images_chunks = Vec::new();

    for _ in 0..size {
        images_chunks.push(Vec::new());
    }

    for i in 0..images.len() {
        let image = images.pop().unwrap();
        images_chunks[i % size].push(image);
    }

    images_chunks
}

pub async fn find_duplicates(
    left_img: gtk::Image,
    left_img_label: gtk::Label,
    right_img: gtk::Image,
    right_img_label: gtk::Label,
) -> Result<(), sqlx::Error> {
    let database = Database::connect_default().await;
    let mut connection = database.get_connection().await;

    let images = connection.get_duplicates().await?;

    if let Some(images) = images {
        left_img.set_file(Some(images.0.path.as_str()));
        right_img.set_file(Some(images.1.path.as_str()));
        left_img_label.set_label(&images.0.path);
        right_img_label.set_label(&images.1.path);
    } else {
        left_img.set_file(None);
        right_img.set_file(None);
        left_img_label.set_label("");
        right_img_label.set_label("");
    }

    Ok(())
}

pub async fn remove_and_protect_image(image_to_protect: &String, image_to_remove: Option<&String>) {
    if let Some(image_to_remove) = image_to_remove {
        fs::remove_file(image_to_remove).unwrap();
    }

    let database = Database::connect_default().await;
    let mut connection = database.get_connection().await;

    connection.mark_protected(&image_to_protect).await.unwrap();
}
