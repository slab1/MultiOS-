//! Multi-desktop workspace management

use crate::*;

/// Represents a virtual desktop workspace
#[derive(Debug)]
pub struct Workspace {
    name: String,
    windows: Vec<WindowId>,
}

impl Workspace {
    /// Create a new workspace with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            windows: Vec::new(),
        }
    }

    /// Get workspace name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set workspace name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Add window to workspace
    pub fn add_window(&mut self, window_id: WindowId) {
        if !self.windows.contains(&window_id) {
            self.windows.push(window_id);
        }
    }

    /// Remove window from workspace
    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.retain(|&id| id != window_id);
    }

    /// Get all windows in workspace
    pub fn windows(&self) -> &[WindowId] {
        &self.windows
    }

    /// Check if workspace contains window
    pub fn contains_window(&self, window_id: WindowId) -> bool {
        self.windows.contains(&window_id)
    }

    /// Get window count
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Check if workspace is empty
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
}

/// Workspace manager handles multiple virtual desktops
#[derive(Debug)]
pub struct WorkspaceManager {
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    max_workspaces: usize,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new(max_workspaces: usize) -> Self {
        let mut workspaces = Vec::new();
        workspaces.push(Workspace::new("Desktop 1"));

        Self {
            workspaces,
            active_workspace: 0,
            max_workspaces,
        }
    }

    /// Create a new workspace with auto-generated name
    pub fn create_workspace(&mut self) -> Result<usize, WorkspaceError> {
        if self.workspaces.len() >= self.max_workspaces {
            return Err(WorkspaceError::MaxWorkspacesReached);
        }

        let workspace_id = self.workspaces.len();
        let name = format!("Desktop {}", workspace_id + 1);
        self.workspaces.push(Workspace::new(name));
        Ok(workspace_id)
    }

    /// Create a named workspace
    pub fn create_named_workspace(&mut self, name: String) -> Result<usize, WorkspaceError> {
        if self.workspaces.len() >= self.max_workspaces {
            return Err(WorkspaceError::MaxWorkspacesReached);
        }

        let workspace_id = self.workspaces.len();
        self.workspaces.push(Workspace::new(name));
        Ok(workspace_id)
    }

    /// Delete a workspace (cannot delete the last workspace)
    pub fn delete_workspace(&mut self, workspace_id: usize) -> Result<(), WorkspaceError> {
        if workspace_id >= self.workspaces.len() {
            return Err(WorkspaceError::InvalidWorkspace);
        }

        if self.workspaces.len() <= 1 {
            return Err(WorkspaceError::CannotDeleteLastWorkspace);
        }

        // Move windows from deleted workspace to active workspace
        let windows_to_move = self.workspaces[workspace_id].windows.clone();
        self.workspaces.remove(workspace_id);

        // Adjust active workspace index if necessary
        if self.active_workspace >= workspace_id && self.active_workspace > 0 {
            self.active_workspace -= 1;
        }

        // Add moved windows to new active workspace
        for window_id in windows_to_move {
            self.add_window_to_workspace(window_id, self.active_workspace);
        }

        Ok(())
    }

    /// Switch to a workspace
    pub fn switch_to_workspace(&mut self, workspace_id: usize) -> Result<(), WorkspaceError> {
        if workspace_id >= self.workspaces.len() {
            return Err(WorkspaceError::InvalidWorkspace);
        }

        self.active_workspace = workspace_id;
        Ok(())
    }

    /// Get active workspace ID
    pub fn active_workspace_id(&self) -> usize {
        self.active_workspace
    }

    /// Get active workspace reference
    pub fn active_workspace(&self) -> &Workspace {
        &self.workspaces[self.active_workspace]
    }

    /// Get active workspace mutable reference
    pub fn active_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace]
    }

    /// Get workspace by ID
    pub fn get_workspace(&self, workspace_id: usize) -> Option<&Workspace> {
        self.workspaces.get(workspace_id)
    }

    /// Get all workspaces
    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    /// Add window to workspace
    pub fn add_window_to_workspace(&mut self, window_id: WindowId, workspace_id: usize) -> Result<(), WorkspaceError> {
        if workspace_id >= self.workspaces.len() {
            return Err(WorkspaceError::InvalidWorkspace);
        }

        // Remove from all workspaces first
        self.remove_window_from_all_workspaces(window_id);

        // Add to target workspace
        self.workspaces[workspace_id].add_window(window_id);
        Ok(())
    }

    /// Remove window from workspace
    pub fn remove_window_from_workspace(&mut self, window_id: WindowId, workspace_id: usize) -> Result<(), WorkspaceError> {
        if workspace_id >= self.workspaces.len() {
            return Err(WorkspaceError::InvalidWorkspace);
        }

        self.workspaces[workspace_id].remove_window(window_id);
        Ok(())
    }

    /// Remove window from all workspaces
    pub fn remove_window_from_all_workspaces(&mut self, window_id: WindowId) {
        for workspace in &mut self.workspaces {
            workspace.remove_window(window_id);
        }
    }

    /// Move window to another workspace
    pub fn move_window_to_workspace(&mut self, window_id: WindowId, target_workspace: usize) -> Result<(), WorkspaceError> {
        if target_workspace >= self.workspaces.len() {
            return Err(WorkspaceError::InvalidWorkspace);
        }

        self.remove_window_from_all_workspaces(window_id);
        self.workspaces[target_workspace].add_window(window_id);
        Ok(())
    }

    /// Get workspace containing window
    pub fn find_window_workspace(&self, window_id: WindowId) -> Option<usize> {
        for (i, workspace) in self.workspaces.iter().enumerate() {
            if workspace.contains_window(window_id) {
                return Some(i);
            }
        }
        None
    }

    /// Get window count for workspace
    pub fn get_window_count(&self, workspace_id: usize) -> Result<usize, WorkspaceError> {
        if workspace_id >= self.workspaces.len() {
            return Err(WorkspaceError::InvalidWorkspace);
        }
        Ok(self.workspaces[workspace_id].window_count())
    }

    /// Get total workspace count
    pub fn workspace_count(&self) -> usize {
        self.workspaces.len()
    }

    /// Get maximum workspace limit
    pub fn max_workspaces(&self) -> usize {
        self.max_workspaces
    }

    /// Check if can create more workspaces
    pub fn can_create_more(&self) -> bool {
        self.workspaces.len() < self.max_workspaces
    }
}

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    InvalidWorkspace,
    MaxWorkspacesReached,
    CannotDeleteLastWorkspace,
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::InvalidWorkspace => write!(f, "Invalid workspace ID"),
            WorkspaceError::MaxWorkspacesReached => write!(f, "Maximum number of workspaces reached"),
            WorkspaceError::CannotDeleteLastWorkspace => write!(f, "Cannot delete the last remaining workspace"),
        }
    }
}

impl std::error::Error for WorkspaceError {}
