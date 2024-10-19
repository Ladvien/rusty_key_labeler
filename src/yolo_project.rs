use bevy::prelude::Resource;
use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fs, path::PathBuf};

use crate::settings::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLabelPair {
    pub name: String,
    pub image_path: Option<String>,
    pub label_path: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairingResult {
    Valid(ImageLabelPair),
    Warning(ImageLabelPair),
    Error(Vec<ImageLabelPair>),
}

#[derive(Debug, Clone)]
pub struct ValidationResults {
    pub valid_results: Vec<PairingResult>,
    pub invalid_results: Vec<PairingResult>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PathWithKey {
    pub path: PathBuf,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloProjectData {
    pub stems: Vec<String>,
    pub pairs: HashMap<String, Vec<PairingResult>>,
}

#[derive(Debug, Resource, Clone)]
pub struct YoloProject {
    pub data: YoloProjectData,
}

impl YoloProject {
    pub fn new(config: &Config) -> Self {
        let image_paths = Self::get_filepaths_for_extension(
            &config.image_path,
            vec!["jpg", "png", "PNG", "JPEG"],
        );
        let label_paths = Self::get_filepaths_for_extension(&config.label_path, vec!["txt"]);

        let all_filepaths = image_paths
            .iter()
            .chain(label_paths.iter())
            .collect::<Vec<&PathWithKey>>();

        let mut stems = Self::get_file_stems(&all_filepaths);

        stems.sort();
        stems.dedup();

        let pairs = Self::pair_images_and_labels(stems.clone(), label_paths, image_paths);

        Self {
            data: YoloProjectData { stems, pairs },
        }
    }

    fn get_filepaths_for_extension(path: &str, extensions: Vec<&str>) -> Vec<PathWithKey> {
        let file_paths = std::fs::read_dir(path).unwrap();
        let mut paths = Vec::<PathWithKey>::new();

        for file_path in file_paths {
            let file_path = file_path.unwrap().path();

            if file_path.is_dir() {
                let filepaths = Self::get_filepaths_for_extension(
                    file_path.to_str().unwrap(),
                    extensions.clone(),
                );

                paths.extend(filepaths);
            }

            if let Some(file_extension) = file_path.extension() {
                let stem = file_path.file_stem().unwrap().to_str().unwrap();
                // TODO: Convert to return a PathWithKey
                let extension_str = file_extension.to_str().unwrap();

                if extensions.contains(&extension_str) {
                    paths.push(PathWithKey {
                        path: file_path.clone(),
                        key: String::from(stem),
                    });
                }
            }
        }

        paths
    }

    fn get_file_stems(filenames: &[&PathWithKey]) -> Vec<String> {
        filenames
            .iter()
            .map(|filename| filename.key.clone())
            .collect::<Vec<String>>()
    }

    fn pair_images_and_labels(
        stems: Vec<String>,
        label_filenames: Vec<PathWithKey>,
        image_filenames: Vec<PathWithKey>,
    ) -> HashMap<String, Vec<PairingResult>> {
        let mut pairing_map = HashMap::<String, Vec<PairingResult>>::new();

        for stem in stems {
            let image_paths_for_stem = image_filenames
                .clone()
                .into_iter()
                .filter(|image| image.key == *stem)
                .map(|image| match image.clone().path.to_str() {
                    Some(path) => Ok(path.to_string()),
                    None => Err(()),
                })
                .collect::<Vec<Result<String, ()>>>();

            let label_paths_for_stem = label_filenames
                .clone()
                .into_iter()
                .filter(|label| label.key == *stem)
                .map(|label| match label.clone().path.to_str() {
                    Some(path) => Ok(path.to_string()),
                    None => Err(()),
                })
                .collect::<Vec<Result<String, ()>>>();

            let unconfirmed_pairs = image_paths_for_stem
                .into_iter()
                .zip_longest(label_paths_for_stem.into_iter());

            // TODO: Peek in label file to determine it is valid.
            // TODO: Filter out invalid labels before pairing.

            pairing_map.insert(
                stem.clone(),
                unconfirmed_pairs
                    .into_iter()
                    .map(|pair| Self::evaluate_pair(stem.clone(), pair))
                    .collect::<Vec<PairingResult>>(),
            );
        }

        pairing_map
    }

    fn evaluate_pair(stem: String, pair: EitherOrBoth<Result<String, ()>>) -> PairingResult {
        match pair {
            EitherOrBoth::Both(image_path, label_path) => match (image_path, label_path) {
                (Ok(image_path), Ok(label_path)) => PairingResult::Valid(ImageLabelPair {
                    name: stem,
                    image_path: Some(image_path),
                    label_path: Some(label_path),
                    message: None,
                }),
                (Ok(image_path), Err(_)) => PairingResult::Warning(ImageLabelPair {
                    name: stem,
                    image_path: Some(image_path),
                    label_path: None,
                    message: Some("Label file is missing.".to_string()),
                }),
                (Err(_), Ok(label_path)) => PairingResult::Warning(ImageLabelPair {
                    name: stem,
                    image_path: None,
                    label_path: Some(label_path),
                    message: Some("Image file is missing.".to_string()),
                }),
                (Err(_), Err(_)) => PairingResult::Error(vec![ImageLabelPair {
                    name: stem,
                    image_path: None,
                    label_path: None,
                    message: Some("Both image and label files are missing.".to_string()),
                }]),
            },
            EitherOrBoth::Left(image_path) => PairingResult::Error(vec![ImageLabelPair {
                name: stem,
                image_path: Some(image_path.unwrap()),
                label_path: None,
                message: Some("Label file is missing.".to_string()),
            }]),
            EitherOrBoth::Right(label_path) => PairingResult::Error(vec![ImageLabelPair {
                name: stem,
                image_path: None,
                label_path: Some(label_path.unwrap()),
                message: Some("Image file is missing.".to_string()),
            }]),
        }
    }

    pub fn validate(
        &self,
    ) -> Result<(Vec<ImageLabelPair>, Vec<ImageLabelPair>), Box<dyn std::error::Error>> {
        // 1. Check if file has a matching image.
        // 2. Check if the file is duplicated
        // 3. Check if file is empty
        // 4. Check if file meets YOLO formatting
        let mut valid_image_label_pairs = Vec::<ImageLabelPair>::new();
        let mut invalid_image_label_pairs = Vec::<ImageLabelPair>::new();

        let data_json = serde_json::to_string(&self.data).unwrap();
        fs::write("validation.json", data_json)?;

        // for (stem, results) in &self.pairs {
        //     for result in results {
        //         match result {
        //             PairingResult::Valid(image_label_pair) => todo!(),
        //             PairingResult::Warning(image_label_pair) => todo!(),
        //             PairingResult::Error(vec) => todo!(),
        //         }
        //     }
        // }

        Ok((valid_image_label_pairs, invalid_image_label_pairs))
    }

    pub fn get_valid_pairs(&self) -> Vec<ImageLabelPair> {
        let mut valid_pairs = Vec::<ImageLabelPair>::new();

        for pair in &self.data.pairs {
            for result in pair.1 {
                if let PairingResult::Valid(image_label_pair) = result {
                    valid_pairs.push(image_label_pair.clone());
                }
            }
        }

        valid_pairs
    }

    pub fn get_invalid_pairs(&self) -> Vec<ImageLabelPair> {
        let mut invalid_pairs = Vec::<ImageLabelPair>::new();

        for pair in &self.data.pairs {
            for result in pair.1 {
                if let PairingResult::Error(vec) = result {
                    invalid_pairs.extend(vec.clone());
                }
            }
        }

        invalid_pairs
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use image::{ImageBuffer, ImageReader, Rgb};
    use rstest::{fixture, rstest};

    #[fixture]
    fn config() -> Config {
        let config: Config = serde_yml::from_str(
            r#"
        image_path: test_output/
        label_path: test_output/
        output_path: output/

        output_format:
            type: yolo
            project_name: test_project
            folder_paths:
                train: ./train/
                validation: ./validation/
                test: ./test/
            class_map:
                0: 'first_class'
                1: 'not_first_class'
        "#,
        )
        .expect("Unable to parse YAML");

        config
    }

    #[fixture]
    fn project(config: Config) -> YoloProject {
        YoloProject::new(&config)
    }

    #[fixture]
    fn image_data() -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        //! An example of generating julia fractals.
        let imgx = 800;
        let imgy = 800;

        let scalex = 3.0 / imgx as f32;
        let scaley = 3.0 / imgy as f32;

        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let r = (0.3 * x as f32) as u8;
            let b = (0.3 * y as f32) as u8;
            *pixel = image::Rgb([r, 0, b]);
        }

        // A redundant loop to demonstrate reading image data
        for x in 0..imgx {
            for y in 0..imgy {
                let cx = y as f32 * scalex - 1.5;
                let cy = x as f32 * scaley - 1.5;

                let c = num_complex::Complex::new(-0.4, 0.6);
                let mut z = num_complex::Complex::new(cx, cy);

                let mut i = 0;
                while i < 255 && z.norm() <= 2.0 {
                    z = z * z + c;
                    i += 1;
                }

                let pixel = imgbuf.get_pixel_mut(x, y);
                let image::Rgb(data) = *pixel;
                *pixel = image::Rgb([data[0], i as u8, data[2]]);
            }
        }

        imgbuf
    }

    fn create_dir_and_write_file(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).expect("Unable to create directory");
        fs::write(path, content).expect("Unable to write file");
    }

    fn create_image_file(path: &Path, image_data: &ImageBuffer<Rgb<u8>, Vec<u8>>) {
        fs::create_dir_all(path.parent().unwrap()).expect("Unable to create directory");
        image_data.save(path).expect("Unable to write file");
    }

    #[rstest]
    fn test_get_filepaths_for_extension(config: Config) {
        let file1 = PathBuf::from("test_output/test_files/test1.txt");
        create_dir_and_write_file(&file1, "Hello, world!");

        let path = file1.parent().unwrap().to_str().unwrap();
        let extensions = vec!["txt"];
        let filepaths = YoloProject::get_filepaths_for_extension(path, extensions);

        assert_eq!(filepaths.len(), 1);
    }

    /*
    Test Scenarios
        Type
        Error = E
        Warn  = W
        Valid = V
        Mixed = M

                 | 1 Label | No Label | Label >2
        1 Image  |  V      |   E      |  M
        No Image |  E      |   -      |  M
        Image >2 |  M      |   E      |  V
     */

    #[rstest]
    fn test_project_validation_produces_one_valid_pair_for_one_image_one_label(
        project: YoloProject,
        image_data: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) {
        let test_output_path = "test_output/test_files1";
        let image_file = PathBuf::from(format!("{}/test1.jpg", test_output_path));
        create_image_file(&image_file, &image_data);

        let file1 = PathBuf::from(format!("{}/test1.txt", test_output_path));
        create_dir_and_write_file(&file1, "Hello, world!");

        let valid_pairs = project.get_valid_pairs();
        let invalid_pairs = project.get_invalid_pairs();

        let valid_pair = valid_pairs
            .into_iter()
            .find(|pair| pair.name == "test1")
            .unwrap();

        let invalid_pair = invalid_pairs
            .into_iter()
            .find(|pair| pair.name == "test1")
            .unwrap();

        assert!(valid_pair.name == invalid_pair.name);
    }

    #[rstest]
    fn test_project_validation_produces_one_invalid_pair_for_one_image_no_label(
        project: YoloProject,
        image_data: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) {
        let test_output_path = "test_output/test_files2";
        let image_file = PathBuf::from(format!("{}/test2.jpg", test_output_path));
        create_image_file(&image_file, &image_data);

        let invalid_pairs = project.get_invalid_pairs();
        let invalid_pair = invalid_pairs
            .into_iter()
            .find(|pair| pair.name == "test2")
            .unwrap();

        assert_eq!(invalid_pair.name, "test2");
    }

    #[rstest]
    fn test_project_validation_produces_one_valid_pair_for_one_image_two_labels(
        project: YoloProject,
        image_data: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) {
        let test_output_path = "test_output/test_files3";
        let image_file = PathBuf::from(format!("{}/test3.jpg", test_output_path));
        create_image_file(&image_file, &image_data);

        let file2 = PathBuf::from(format!("{}/dir1/test3.txt", test_output_path));
        let file3 = PathBuf::from(format!("{}/dir2/test3.txt", test_output_path));
        create_dir_and_write_file(&file2, "Hello, world!");
        create_dir_and_write_file(&file3, "Hello, world!");

        let valid_pairs = project.get_valid_pairs();
        let invalid_pairs = project.get_invalid_pairs();

        let valid_pair = valid_pairs
            .into_iter()
            .find(|pair| pair.name == "test3")
            .unwrap();

        let invalid_pair = invalid_pairs
            .into_iter()
            .find(|pair| pair.name == "test3")
            .unwrap();

        assert!(valid_pair.name == invalid_pair.name);
    }

    #[rstest]
    fn test_project_validation_produces_one_invalid_pair_for_no_image_one_label(
        project: YoloProject,
    ) {
        let test_output_path = "test_output/test_files4";
        let file1 = PathBuf::from(format!("{}/test4.txt", test_output_path));
        create_dir_and_write_file(&file1, "Hello, world!");

        let invalid_pairs = project.get_invalid_pairs();
        let invalid_pair = invalid_pairs
            .into_iter()
            .find(|pair| pair.name == "test4")
            .unwrap();

        assert_eq!(invalid_pair.name, "test4");
    }

    #[rstest]
    fn test_project_validation_produces_one_invalid_pair_for_no_image_no_label() {
        let config: Config = serde_yml::from_str(
            r#"
                image_path: test_output/
                label_path: test_output/
                output_path: output/

                output_format:
                    type: yolo
                    project_name: test_project
                    folder_paths:
                        train: ./train/
                        validation: ./validation/
                        test: ./test/
                    class_map:
                        0: 'first_class'
                        1: 'not_first_class'
                "#,
        )
        .unwrap();

        let project = YoloProject::new(&config);

        let test_output_path = "test_output/test_files5";
        // Make the directory
        let file1 = PathBuf::from(format!("{}/test5.txt", test_output_path));
        fs::create_dir_all(file1.parent().unwrap()).expect("Unable to create directory");

        let invalid_pairs = project.get_invalid_pairs();

        assert!(invalid_pairs.is_empty())
    }
}
