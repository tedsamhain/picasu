use std::path::Path;

pub fn run_tui(
    root: &Path,
    status_filter: Option<&str>,
    type_filter: Option<&str>,
    priority_filter: Option<&str>,
    area_filter: Option<&str>,
    search_query: Option<&str>,
) {
    let tasks_dir = root.join(".plan").join("tasks");
    let entries = crate::plan::read_task_files(&tasks_dir).unwrap_or_default();
    let tasks = crate::plan::load_and_filter_tasks(
        &entries,
        status_filter,
        type_filter,
        priority_filter,
        area_filter,
        search_query,
    );

    if tasks.is_empty() {
        eprintln!("(no tasks match filters)");
        return;
    }

    eprintln!(
        "plan tui: {} tasks loaded (TUI not yet implemented)",
        tasks.len()
    );
}
