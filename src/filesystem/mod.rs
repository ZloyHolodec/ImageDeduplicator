use std::fs;

pub fn find_file_recursive(path: String, extensions: &Vec<String>) -> Vec<String> {
    let dir_data = fs::read_dir(path);

    let mut result = Vec::new();

    if let Ok(dir_data) = dir_data {
        let dir_data: Vec<fs::DirEntry> = dir_data.filter_map(|x| x.ok()).collect();

        for folder_value in dir_data {
            let path = folder_value.path();
            let path_str = path.to_str();
            if let Some(path_str) = path_str {
                let path_str = path_str.to_string();
                if path.is_file() && is_correct_extension(&path_str, extensions) {
                    result.push(path_str.to_string());
                } else if path.is_dir() {
                    let mut files = find_file_recursive(path_str, extensions);
                    result.append(&mut files);
                }
            }
        }
    }

    result
}

fn is_correct_extension(path: &String, extensions: &Vec<String>) -> bool {
    for extension in extensions.iter() {
        if path.ends_with(extension) {
            return true;
        }
    }

    false
}

#[test]
fn test_find_folders() {
    let extensions = vec![".jpg".to_string()];
    let result = find_file_recursive("tests/test_folders".to_string(), &extensions);

    assert_eq!(5, result.len());
}
