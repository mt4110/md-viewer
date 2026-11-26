#[tauri::command]
pub(crate) fn app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub(crate) async fn check_update(
    app: tauri::AppHandle,
) -> Result<md_viewer::update::UpdateCheck, md_viewer::update::UpdateError> {
    md_viewer::update::check(&app.package_info().version).await
}
