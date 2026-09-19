pub mod commands;

pub mod error;

pub mod state;

use commands::{
    analyze_quality,
    analyze_statistics,
    apply_cleaning,
    compare_dataset_quality,
    export_dataset_csv,
    export_dataset_xlsx,
    generate_insights,
    get_chart_data,
    get_compatible_charts,
    open_dataset,
    profile_dataset,
    recommend_charts,
    suggest_cleaning,
};

use state::DatasetState;

#[cfg_attr(
    mobile,
    tauri::mobile_entry_point
)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_opener::init(),
        )
        .plugin(
            tauri_plugin_dialog::init(),
        )
        .manage(
            DatasetState::new(),
        )
        .invoke_handler(
            tauri::generate_handler![
                open_dataset,
                profile_dataset,
                suggest_cleaning,
                apply_cleaning,
                analyze_quality,
                compare_dataset_quality,
                analyze_statistics,
                generate_insights,
                recommend_charts,
                get_compatible_charts,
                get_chart_data,
                export_dataset_csv,
                export_dataset_xlsx,
            ],
        )
        .run(
            tauri::generate_context!(),
        )
        .expect(
            "error while running Flowtren",
        );
}