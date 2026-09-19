use std::{
    path::{
        Path,
        PathBuf,
    },
    sync::Mutex,
};

use flowtren_cleaner::suggestion::CleaningSuggestion;
use polars::prelude::DataFrame;

#[derive(Debug)]
pub struct LoadedDataset {
    pub original_dataframe: DataFrame,

    pub dataframe: DataFrame,

    pub path: PathBuf,

    pub file_name: String,

    pub cleaning_suggestions:
        Vec<CleaningSuggestion>,
}

impl LoadedDataset {
    pub fn new(
        dataframe: DataFrame,
        path: PathBuf,
        file_name: String,
    ) -> Self {
        Self {
            original_dataframe:
                dataframe.clone(),

            dataframe,

            path,

            file_name,

            cleaning_suggestions:
                Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DatasetState {
    dataset:
        Mutex<Option<LoadedDataset>>,
}

impl DatasetState {
    pub fn new() -> Self {
        Self {
            dataset:
                Mutex::new(None),
        }
    }

    pub fn replace(
        &self,
        dataset: LoadedDataset,
    ) -> Result<(), String> {
        let mut guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        *guard =
            Some(dataset);

        Ok(())
    }

    pub fn with_dataset<T, F>(
        &self,
        operation: F,
    ) -> Result<T, String>
    where
        F: FnOnce(
            &LoadedDataset,
        ) -> Result<T, String>,
    {
        let guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_ref()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        operation(
            dataset,
        )
    }

    pub fn with_dataset_mut<T, F>(
        &self,
        operation: F,
    ) -> Result<T, String>
    where
        F: FnOnce(
            &mut LoadedDataset,
        ) -> Result<T, String>,
    {
        let mut guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_mut()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        operation(
            dataset,
        )
    }

    pub fn dataframe_snapshot(
        &self,
    ) -> Result<DataFrame, String> {
        let guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_ref()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        Ok(
            dataset
                .dataframe
                .clone(),
        )
    }

    pub fn profile_snapshot(
        &self,
    ) -> Result<
        (
            DataFrame,
            String,
        ),
        String,
    > {
        let guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_ref()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        Ok((
            dataset
                .dataframe
                .clone(),

            dataset
                .file_name
                .clone(),
        ))
    }

    pub fn quality_comparison_snapshot(
        &self,
    ) -> Result<
        (
            DataFrame,
            DataFrame,
        ),
        String,
    > {
        let guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_ref()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        Ok((
            dataset
                .original_dataframe
                .clone(),

            dataset
                .dataframe
                .clone(),
        ))
    }

    pub fn cleaning_snapshot(
        &self,
    ) -> Result<
        (
            DataFrame,
            PathBuf,
        ),
        String,
    > {
        let guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_ref()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        Ok((
            dataset
                .dataframe
                .clone(),

            dataset
                .path
                .clone(),
        ))
    }

    pub fn store_cleaning_suggestions(
        &self,
        expected_path: &Path,
        suggestions:
            Vec<CleaningSuggestion>,
    ) -> Result<(), String> {
        let mut guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        let dataset =
            guard
                .as_mut()
                .ok_or_else(
                    || {
                        "No dataset is currently loaded"
                            .to_string()
                    },
                )?;

        if dataset.path
            != expected_path
        {
            return Err(
                "Dataset changed while cleaning suggestions were being analyzed"
                    .to_string(),
            );
        }

        dataset.cleaning_suggestions =
            suggestions;

        Ok(())
    }

    pub fn clear(
        &self,
    ) -> Result<(), String> {
        let mut guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        *guard =
            None;

        Ok(())
    }

    pub fn is_loaded(
        &self,
    ) -> Result<bool, String> {
        let guard =
            self.dataset
                .lock()
                .map_err(
                    |_| {
                        "Dataset state lock is poisoned"
                            .to_string()
                    },
                )?;

        Ok(
            guard.is_some(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use polars::prelude::*;

    fn sample_dataframe()
        -> DataFrame
    {
        DataFrame::new_infer_height(
            vec![
                Column::new(
                    "name".into(),
                    [
                        "Alice",
                        "Bob",
                    ],
                ),

                Column::new(
                    "age".into(),
                    [
                        20_i64,
                        30_i64,
                    ],
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn state_starts_empty() {
        let state =
            DatasetState::new();

        assert!(
            !state
                .is_loaded()
                .unwrap()
        );
    }

    #[test]
    fn stores_dataset() {
        let state =
            DatasetState::new();

        state
            .replace(
                LoadedDataset::new(
                    sample_dataframe(),

                    PathBuf::from(
                        "test.csv",
                    ),

                    "test.csv"
                        .to_string(),
                ),
            )
            .unwrap();

        assert!(
            state
                .is_loaded()
                .unwrap()
        );
    }

    #[test]
    fn preserves_original_dataframe() {
        let state =
            DatasetState::new();

        state
            .replace(
                LoadedDataset::new(
                    sample_dataframe(),

                    PathBuf::from(
                        "test.csv",
                    ),

                    "test.csv"
                        .to_string(),
                ),
            )
            .unwrap();

        let (
            original_rows,
            current_rows,
        ) =
            state
                .with_dataset(
                    |dataset| {
                        Ok((
                            dataset
                                .original_dataframe
                                .height(),

                            dataset
                                .dataframe
                                .height(),
                        ))
                    },
                )
                .unwrap();

        assert_eq!(
            original_rows,
            current_rows
        );
    }

    #[test]
    fn creates_dataframe_snapshot() {
        let state =
            DatasetState::new();

        state
            .replace(
                LoadedDataset::new(
                    sample_dataframe(),

                    PathBuf::from(
                        "test.csv",
                    ),

                    "test.csv"
                        .to_string(),
                ),
            )
            .unwrap();

        let snapshot =
            state
                .dataframe_snapshot()
                .unwrap();

        assert_eq!(
            snapshot.height(),
            2
        );

        assert_eq!(
            snapshot.width(),
            2
        );
    }

    #[test]
    fn clears_dataset() {
        let state =
            DatasetState::new();

        state
            .replace(
                LoadedDataset::new(
                    sample_dataframe(),

                    PathBuf::from(
                        "test.csv",
                    ),

                    "test.csv"
                        .to_string(),
                ),
            )
            .unwrap();

        state
            .clear()
            .unwrap();

        assert!(
            !state
                .is_loaded()
                .unwrap()
        );
    }
}