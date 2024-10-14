use bevy::prelude::Resource;
use itertools::{EitherOrBoth, Itertools};
use std::path::PathBuf;

use crate::settings::Config;

#[derive(Debug, Resource)]
pub struct YoloProject {
    pub image_path: String,
    pub label_path: String,
    pub image_label_pairs: Option<Vec<PotentialImageLabelPair>>,
}

#[derive(Debug)]
pub struct PotentialImageLabelPair {
    pub name: String,
    pub image_path: Option<String>,
    pub label_path: Option<String>,
}

#[derive(Debug)]
pub struct InvalidImageLabelPair {
    pub name: String,
    pub image_path: Option<String>,
    pub label_path: Option<String>,
    pub error: String,
}

#[derive(Debug)]
pub struct ValidImageLabelPair {
    pub name: String,
    pub image_path: String,
    pub label_path: String,
}

#[derive(Debug)]
pub struct ValidationResults {
    pub valid_results: Vec<PotentialImageLabelPair>,
    pub invalid_results: Vec<InvalidImageLabelPair>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PathWithKey {
    pub path: PathBuf,
    pub key: String,
}

impl YoloProject {
    pub fn new(config: &Config) -> Self {
        Self {
            image_path: config.image_path.clone(),
            image_label_pairs: None,
            label_path: config.label_path.clone(),
        }
    }

    fn get_filepaths_for_extension(&self, path: &str, extension: &str) -> Vec<PathWithKey> {
        let file_paths = std::fs::read_dir(path).unwrap();
        let mut paths = Vec::<PathWithKey>::new();

        for file_path in file_paths {
            let file_path = file_path.unwrap().path();

            if file_path.is_dir() {
                let filepaths =
                    self.get_filepaths_for_extension(file_path.to_str().unwrap(), extension);

                paths.extend(filepaths);
            }

            if let Some(file_extension) = file_path.extension() {
                let stem = file_path.file_stem().unwrap().to_str().unwrap();
                // TODO: Convert to return a PathWithKey
                if file_extension == extension {
                    paths.push(PathWithKey {
                        path: file_path.clone(),
                        key: String::from(stem),
                    });
                }
            }
        }

        paths
    }

    fn get_file_stems(&self, filenames: Vec<PathWithKey>) -> Vec<String> {
        filenames
            .into_iter()
            .map(|filename| filename.key)
            .collect::<Vec<String>>()
    }

    pub fn pair_images_and_labels(
        &mut self,
        all_stems: Vec<String>,
        label_filenames: Vec<PathWithKey>,
        image_filenames: Vec<PathWithKey>,
    ) -> Vec<PotentialImageLabelPair> {
        let mut potential_pairs = Vec::new();
        for stem in all_stems {
            let image_paths_for_stem = image_filenames
                .clone()
                .into_iter()
                .filter(|image| image.key == stem)
                .collect::<Vec<PathWithKey>>();

            let label_paths_for_stem = label_filenames
                .clone()
                .into_iter()
                .filter(|label| label.key == stem)
                .collect::<Vec<PathWithKey>>();

            let image_label_pairs = image_paths_for_stem
                .into_iter()
                .zip_longest(label_paths_for_stem.into_iter());

            for item in image_label_pairs {
                match item {
                    EitherOrBoth::Both(image, label) => {
                        potential_pairs.push(PotentialImageLabelPair {
                            name: stem.clone(),
                            image_path: Some(image.path.to_str().unwrap().to_string()),
                            label_path: Some(label.path.to_str().unwrap().to_string()),
                        });
                    }
                    EitherOrBoth::Left(image) => {
                        potential_pairs.push(PotentialImageLabelPair {
                            name: stem.clone(),
                            image_path: Some(image.path.to_str().unwrap().to_string()),
                            label_path: None,
                        });
                    }
                    EitherOrBoth::Right(label) => {
                        potential_pairs.push(PotentialImageLabelPair {
                            name: stem.clone(),
                            image_path: None,
                            label_path: Some(label.path.to_str().unwrap().to_string()),
                        });
                    }
                }
            }
        }

        potential_pairs
    }

    fn validate_label_files(
        &self,
        image_filenames: Vec<PathWithKey>,
        label_filenames: Vec<PathWithKey>,
    ) -> Result<(Vec<ValidImageLabelPair>, Vec<InvalidImageLabelPair>), Box<dyn std::error::Error>>
    {
        // 1. Check if file has a matching image.
        // 2. Check if the file is duplicated
        // 3. Check if file is empty
        // 4. Check if file meets YOLO formatting
        let mut valid_image_label_pairs = Vec::<ValidImageLabelPair>::new();
        let mut invalid_image_label_pairs = Vec::<InvalidImageLabelPair>::new();

        for image_path_with_key in image_filenames {
            println!("{:?}", image_path_with_key);
        }

        Ok((valid_image_label_pairs, invalid_image_label_pairs))
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Load all image and label file paths
        // 2. Get all stems of both file types in a master list
        // 3. Run through all stems, create a structure for each stem.
        //    The struct should contain a vector for all label paths that match the stem
        //    and a vector for all image paths that match the stem.
        // 4. Project validation is then started:
        //   a. Check if all stems have at least one image and one label
        //   b. Check if all stems have the same number of images and labels
        //   c. Look for duplicate labels or images, or both
        // 5. For each stem without error, create a new ImageLabelPair and add it to the project

        let image_filenames = self.get_filepaths_for_extension(&self.image_path, "png");
        let label_filenames = self.get_filepaths_for_extension(&self.label_path, "txt");

        println!("Length of image filenames: {}", image_filenames.len());
        println!("Length of label filenames: {}", label_filenames.len());

        let all_stems = self.get_file_stems(label_filenames.clone());

        let mut image_label_pairs =
            self.pair_images_and_labels(all_stems, label_filenames, image_filenames);

        // image_label_pairs.dedup_by(|a, b| a.name == b.name);
        image_label_pairs.sort_by(|a, b| a.name.cmp(&b.name));

        println!("{:#?}", image_label_pairs.first());
        println!("Length of image label pairs: {}", image_label_pairs.len());

        self.image_label_pairs = Some(image_label_pairs);

        Ok(())
    }
}
