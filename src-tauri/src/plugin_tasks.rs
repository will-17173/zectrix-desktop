use std::path::PathBuf;

pub async fn run_plugin_loop_task_tick(data_dir: PathBuf, task_id: i64) -> anyhow::Result<()> {
    let state = crate::state::AppState { data_dir };
    let task = state
        .list_plugin_loop_tasks()?
        .into_iter()
        .find(|task| task.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;

    if task.status != "running" {
        return Ok(());
    }

    let config = task.config.clone().unwrap_or_default();

    if let Err(error) = state
        .push_plugin_once(&task.plugin_kind, &task.plugin_id, &task.device_id, task.page_id, config)
        .await
    {
        let _ = state.mark_plugin_loop_task_error(task_id, &error.to_string());
        return Err(error);
    }

    state.mark_plugin_loop_task_pushed(task_id)?;
    Ok(())
}