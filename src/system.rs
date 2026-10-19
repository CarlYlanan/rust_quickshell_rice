use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::Value;

use crate::model::Workspace;

#[derive(Default)]
pub struct WorkspaceSnapshot {
    pub revision: u64,
    pub workspaces: Vec<Value>,
}

pub struct WorkspaceStream {
    snapshot: Arc<Mutex<WorkspaceSnapshot>>,
    child: Child,
}

impl Drop for WorkspaceStream {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl WorkspaceStream {
    pub fn new() -> Option<Self> {
        let mut child = Command::new("niri")
            .args(["msg", "--json", "event-stream"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdout = child.stdout.take()?;
        let snapshot = Arc::new(Mutex::new(WorkspaceSnapshot::default()));
        let snapshot_for_reader = snapshot.clone();

        let _reader = thread::Builder::new()
            .name("owo-workspaces".into())
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    let Ok(line) = line else { break };
                    if let Ok(mut snap) = snapshot_for_reader.lock() {
                        if apply_workspace_event(&mut snap, &line) {
                            snap.revision = snap.revision.wrapping_add(1);
                        }
                    }
                }
            })
            .ok()?;

        Some(Self { snapshot, child })
    }

    pub fn snapshot(&self) -> Arc<Mutex<WorkspaceSnapshot>> {
        self.snapshot.clone()
    }
}

fn apply_workspace_event(snapshot: &mut WorkspaceSnapshot, line: &str) -> bool {
    let Ok(event) = serde_json::from_str::<Value>(line) else { return false };

    if let Some(workspaces) = event
        .get("WorkspacesChanged")
        .and_then(|e| e.get("workspaces"))
        .and_then(Value::as_array)
    {
        snapshot.workspaces = workspaces.clone();
        return true;
    }

    if let Some(activated) = event.get("WorkspaceActivated") {
        let Some(id) = activated.get("id").and_then(Value::as_u64) else { return false };
        let focused = activated.get("focused").and_then(Value::as_bool).unwrap_or(false);
        let Some(output) = snapshot
            .workspaces
            .iter()
            .find(|w| w.get("id").and_then(Value::as_u64) == Some(id))
            .and_then(|w| w.get("output"))
            .and_then(Value::as_str)
            .map(str::to_string)
        else { return false };

        for w in &mut snapshot.workspaces {
            let is_activated = w.get("id").and_then(Value::as_u64) == Some(id);
            if w.get("output").and_then(Value::as_str) == Some(&output) {
                w["is_active"] = Value::Bool(is_activated);
            }
            if focused {
                w["is_focused"] = Value::Bool(is_activated);
            }
        }
        return true;
    }

    if let Some(changed) = event.get("WorkspaceActiveWindowChanged") {
        let Some(id) = changed.get("workspace_id").and_then(Value::as_u64) else { return false };
        let Some(w) = snapshot.workspaces.iter_mut().find(|w| {
            w.get("id").and_then(Value::as_u64) == Some(id)
        }) else { return false };
        if let Some(active_window_id) = changed.get("active_window_id") {
            w["active_window_id"] = active_window_id.clone();
        }
        return true;
    }

    false
}

pub fn workspace_states_for_output(output_name: &str, workspaces: &[Value]) -> Vec<Workspace> {
    let mut states: Vec<Workspace> = workspaces
        .iter()
        .filter(|w| w.get("output").and_then(Value::as_str) == Some(output_name))
        .filter_map(|w| {
            let id = w.get("idx").and_then(Value::as_u64).and_then(|i| u8::try_from(i).ok())?;
            Some(Workspace {
                id,
                active: w.get("is_active").and_then(Value::as_bool).unwrap_or(false),
                focused: w.get("is_focused").and_then(Value::as_bool).unwrap_or(false),
                occupied: w.get("active_window_id").is_some_and(|v| !v.is_null()),
            })
        })
        .collect();
    states.sort_by_key(|w| w.id);
    states
}

pub fn focused_output_name() -> Option<String> {
    let output = Command::new("niri")
        .args(["msg", "--json", "focused-output"])
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    serde_json::from_slice::<Value>(&output.stdout)
        .ok()?
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub fn refresh_workspaces_from_niri(output_name: &str) -> Vec<Workspace> {
    let Ok(output) = Command::new("niri")
        .args(["msg", "--json", "workspaces"])
        .output()
    else { return Vec::new() };
    if !output.status.success() { return Vec::new(); }
    let Ok(workspaces) = serde_json::from_slice::<Vec<Value>>(&output.stdout) else { return Vec::new() };
    workspace_states_for_output(output_name, &workspaces)
}