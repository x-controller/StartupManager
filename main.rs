#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tauri::{
    State,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
    Manager,
    WindowEvent,
};

// ── Data Models ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub command: String,
    pub description: String,
    pub color: String,
    #[serde(default)]
    pub port: String,
    #[serde(default)]
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectStatus {
    pub id: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub output: String,
}

// ── App State ────────────────────────────────────────────────

pub struct AppState {
    processes: Mutex<HashMap<String, Child>>,
    output_logs: Arc<Mutex<HashMap<String, String>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            output_logs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// ── Config File Path ─────────────────────────────────────────

fn get_config_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".config"))
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    }
}

fn get_config_path() -> std::path::PathBuf {
    let mut path = get_config_dir();
    path.push("startup-manager");
    fs::create_dir_all(&path).ok();
    path.push("projects.json");
    path
}

// ── Load / Save ──────────────────────────────────────────────

fn load_projects() -> Vec<Project> {
    let path = get_config_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_projects(projects: &[Project]) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(projects) {
        fs::write(&path, data).ok();
    }
}

// ── Tauri Commands ───────────────────────────────────────────

#[tauri::command]
fn get_projects() -> Vec<Project> {
    load_projects()
}

#[tauri::command]
fn add_project(
    name: String,
    directory: String,
    command: String,
    description: String,
    color: String,
    port: String,
    url: String,
) -> Vec<Project> {
    let mut projects = load_projects();
    let id = format!("proj_{}", timestamp_id());
    let project = Project {
        id,
        name,
        directory,
        command,
        description,
        color,
        port,
        url,
        created_at: timestamp_id(),
    };
    projects.push(project);
    save_projects(&projects);
    projects
}

#[tauri::command]
fn update_project(
    id: String,
    name: String,
    directory: String,
    command: String,
    description: String,
    color: String,
    port: String,
    url: String,
) -> Vec<Project> {
    let mut projects = load_projects();
    if let Some(p) = projects.iter_mut().find(|p| p.id == id) {
        p.name = name;
        p.directory = directory;
        p.command = command;
        p.description = description;
        p.color = color;
        p.port = port;
        p.url = url;
    }
    save_projects(&projects);
    projects
}

#[tauri::command]
fn delete_project(id: String, state: State<AppState>) -> Vec<Project> {
    let _ = stop_project(id.clone(), state);
    let mut projects = load_projects();
    projects.retain(|p| p.id != id);
    save_projects(&projects);
    projects
}

#[tauri::command]
fn start_project(id: String, state: State<AppState>) -> Result<ProjectStatus, String> {
    let projects = load_projects();
    let project = projects
        .iter()
        .find(|p| p.id == id)
        .ok_or("Project not found")?;

    {
        let processes = state.processes.lock().map_err(|e| e.to_string())?;
        if processes.contains_key(&id) {
            return Err("Project is already running".into());
        }
    }

    let mut child = spawn_process(&project.directory, &project.command)?;
    let pid = child.id();

    // Initialize log
    {
        let mut logs = state.output_logs.lock().map_err(|e| e.to_string())?;
        logs.insert(
            id.clone(),
            format!("[Started] PID: {} | {}\n", pid, project.command),
        );
    }

    // Capture stdout in background thread
    let logs_ref = Arc::clone(&state.output_logs);
    let id_clone = id.clone();
    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = strip_ansi(&line);
                    if let Ok(mut logs) = logs_ref.lock() {
                        if let Some(log) = logs.get_mut(&id_clone) {
                            log.push_str(&line);
                            log.push('\n');
                            // Keep last 100KB to avoid unbounded growth
                            if log.len() > 102400 {
                                let truncated = log.split_off(log.len() - 81920);
                                *log = format!("[...truncated...]\n{}", truncated);
                            }
                        }
                    }
                }
            }
        });
    }

    // Capture stderr in background thread
    let logs_ref2 = Arc::clone(&state.output_logs);
    let id_clone2 = id.clone();
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = strip_ansi(&line);
                    if let Ok(mut logs) = logs_ref2.lock() {
                        if let Some(log) = logs.get_mut(&id_clone2) {
                            log.push_str("[ERR] ");
                            log.push_str(&line);
                            log.push('\n');
                            if log.len() > 102400 {
                                let truncated = log.split_off(log.len() - 81920);
                                *log = format!("[...truncated...]\n{}", truncated);
                            }
                        }
                    }
                }
            }
        });
    }

    {
        let mut processes = state.processes.lock().map_err(|e| e.to_string())?;
        processes.insert(id.clone(), child);
    }

    Ok(ProjectStatus {
        id,
        running: true,
        pid: Some(pid),
        output: String::new(),
    })
}

#[tauri::command]
fn stop_project(id: String, state: State<AppState>) -> Result<ProjectStatus, String> {
    let mut processes = state.processes.lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = processes.remove(&id) {
        kill_process_tree(&mut child);

        let mut logs = state.output_logs.lock().map_err(|e| e.to_string())?;
        if let Some(log) = logs.get_mut(&id) {
            log.push_str("[Stopped]\n");
        }
    }

    Ok(ProjectStatus {
        id,
        running: false,
        pid: None,
        output: String::new(),
    })
}

#[tauri::command]
fn get_status(id: String, state: State<AppState>) -> ProjectStatus {
    let mut processes = state.processes.lock().unwrap();
    let logs = state.output_logs.lock().unwrap();

    let running = if let Some(child) = processes.get_mut(&id) {
        match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    } else {
        false
    };

    if !running {
        processes.remove(&id);
    }

    let pid = if running {
        processes.get(&id).map(|c| c.id())
    } else {
        None
    };

    let output = logs.get(&id).cloned().unwrap_or_default();

    ProjectStatus { id, running, pid, output }
}

#[tauri::command]
fn get_all_statuses(state: State<AppState>) -> Vec<ProjectStatus> {
    let projects = load_projects();
    projects
        .iter()
        .map(|p| get_status(p.id.clone(), state.clone()))
        .collect()
}

#[tauri::command]
fn open_directory(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("explorer")
            .arg(&path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_project_logs(id: String, state: State<AppState>) -> String {
    let logs = state.output_logs.lock().unwrap();
    logs.get(&id).cloned().unwrap_or_default()
}

// ── Platform Process Helpers ─────────────────────────────────

#[cfg(target_os = "windows")]
fn spawn_process(directory: &str, command: &str) -> Result<Child, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    Command::new("cmd")
        .args(["/C", &format!("chcp 65001>nul && cd /d {} && {}", directory, command)])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to start: {}", e))
}

#[cfg(not(target_os = "windows"))]
fn spawn_process(directory: &str, command: &str) -> Result<Child, String> {
    Command::new("sh")
        .args(["-c", &format!("cd {} && {}", directory, command)])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start: {}", e))
}

#[cfg(target_os = "windows")]
fn kill_process_tree(child: &mut Child) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let pid = child.id();
    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok();
    child.wait().ok();
}

#[cfg(not(target_os = "windows"))]
fn kill_process_tree(child: &mut Child) {
    child.kill().ok();
    child.wait().ok();
}

// ── Helpers ──────────────────────────────────────────────────

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ESC [ ... final_byte sequences
            if let Some(next) = chars.next() {
                if next == '[' {
                    // Consume until we hit a letter (0x40–0x7E)
                    for ch in chars.by_ref() {
                        if ch.is_ascii_alphabetic() || ch == 'm' {
                            break;
                        }
                    }
                }
                // else: skip the single char after ESC
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn timestamp_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}_{}", d.as_secs(), d.subsec_millis())
}

// ── Main ─────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_projects,
            add_project,
            update_project,
            delete_project,
            start_project,
            stop_project,
            get_status,
            get_all_statuses,
            get_project_logs,
            open_directory,
        ])
        .setup(|app| {
            // 创建托盘菜单
            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // 创建系统托盘图标
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Startup Manager")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 双击托盘图标显示窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        // 拦截窗口关闭事件，改为隐藏
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().ok();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
