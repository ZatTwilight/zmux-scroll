use std::collections::{BTreeMap, HashSet};

use zellij_tile::prelude::{actions::Action, *};
use zellij_utils::position::Position;

macro_rules! debug_log {
    ($plugin:expr, $($arg:tt)*) => {
        if $plugin.debug {
            eprintln!("zmux-scroll: {}", format_args!($($arg)*));
        }
    };
}

#[derive(Default)]
struct ZmuxScroll {
    debug: bool,
    permissions_granted: bool,
    status: String,
    scrolled_panes: HashSet<PaneId>,
    cur_mode: InputMode,
    cur_pane: Option<PaneId>,
    panes: PaneManifest,
    active_tab: Option<usize>,
}

register_plugin!(ZmuxScroll);

#[derive(Debug, Clone, Copy)]
enum CustomAction {
    ScrollAt(Position),
    SwitchMode(InputMode),
    FocusMayHaveChanged,
}

fn pane_at_position(
    manifest: &PaneManifest,
    tab: usize,
    line: isize,
    column: usize,
) -> Option<PaneId> {
    if line < 0 {
        return None;
    }

    let line = line as usize;
    let panes = manifest.panes.get(&tab)?;

    panes.iter().rev().find_map(|pane| {
        let x1 = pane.pane_x;
        let x2 = pane.pane_x + pane.pane_columns;
        let y1 = pane.pane_y;
        let y2 = pane.pane_y + pane.pane_rows;

        if column >= x1 && column < x2 && line >= y1 && line < y2 {
            Some(if pane.is_plugin {
                PaneId::Plugin(pane.id)
            } else {
                PaneId::Terminal(pane.id)
            })
        } else {
            None
        }
    })
}

fn config_bool(configuration: &BTreeMap<String, String>, key: &str) -> bool {
    configuration
        .get(key)
        .map(|value| {
            let value = value.trim();
            value == "1"
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case("on")
        })
        .unwrap_or(false)
}

fn classify_action(action: &Action) -> Option<CustomAction> {
    match action {
        Action::SwitchToMode { input_mode } | Action::SwitchModeForAllClients { input_mode } => {
            Some(CustomAction::SwitchMode(*input_mode))
        }

        Action::ScrollUpAt { position } | Action::ScrollDownAt { position } => {
            Some(CustomAction::ScrollAt(*position))
        }
        Action::MouseEvent { event } if event.wheel_up || event.wheel_down => {
            Some(CustomAction::ScrollAt(event.position))
        }

        // Actions that can change the focused pane and/or active tab. We intentionally over-match
        // a bit here: the cost is one focused-pane query for these actions, while missing one means
        // stale scroll-state restoration.
        Action::FocusNextPane
        | Action::FocusPreviousPane
        | Action::SwitchFocus
        | Action::MoveFocus { .. }
        | Action::MoveFocusOrTab { .. }
        | Action::FocusTerminalPaneWithId { .. }
        | Action::FocusPluginPaneWithId { .. }
        | Action::FocusPaneByPaneId { .. }
        | Action::GoToNextTab
        | Action::GoToPreviousTab
        | Action::GoToTab { .. }
        | Action::GoToTabName { .. }
        | Action::GoToTabById { .. }
        | Action::ToggleTab
        | Action::CloseTab
        | Action::CloseTabById { .. }
        | Action::CloseFocus
        | Action::CloseFocusByPaneId { .. }
        | Action::CloseTerminalPane { .. }
        | Action::ClosePluginPane { .. }
        | Action::NewPane { .. }
        | Action::NewBlockingPane { .. }
        | Action::EditFile { .. }
        | Action::NewFloatingPane { .. }
        | Action::NewTiledPane { .. }
        | Action::NewInPlacePane { .. }
        | Action::NewStackedPane { .. }
        | Action::Run { .. }
        | Action::LaunchOrFocusPlugin { .. }
        | Action::LaunchPlugin { .. }
        | Action::NewTiledPluginPane { .. }
        | Action::NewFloatingPluginPane { .. }
        | Action::NewInPlacePluginPane { .. }
        | Action::TogglePaneEmbedOrFloating
        | Action::TogglePaneEmbedOrFloatingByPaneId { .. }
        | Action::ToggleFocusFullscreen
        | Action::ToggleFocusFullscreenByPaneId { .. }
        | Action::ToggleFloatingPanes
        | Action::ShowFloatingPanes { .. }
        | Action::HideFloatingPanes { .. }
        | Action::BreakPane
        | Action::BreakPaneLeft
        | Action::BreakPaneRight
        | Action::NewTab {
            should_change_focus_to_new_tab: true,
            ..
        }
        | Action::SwitchSession { .. } => Some(CustomAction::FocusMayHaveChanged),

        _ => None,
    }
}

impl ZellijPlugin for ZmuxScroll {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.debug = config_bool(&configuration, "debug");
        subscribe(&[
            EventType::PermissionRequestResult,
            EventType::PaneUpdate,
            EventType::UserAction,
            EventType::ModeUpdate,
        ]);
        request_permission(&[
            PermissionType::InterceptInput,
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
            PermissionType::OpenTerminalsOrPlugins,
        ]);
        debug_log!(self, "loaded; requesting permissions");
        self.status = "zmux-scroll background plugin loaded".to_string();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                debug_log!(self, "permissions granted");
                self.permissions_granted = true;
                self.sync_focused_pane();
                false
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                debug_log!(self, "permissions denied");
                self.permissions_granted = false;
                self.status = "permissions denied".to_string();
                false
            }
            Event::PaneUpdate(panes) => {
                self.panes = panes;
                self.sync_focused_pane();
                false
            }
            Event::UserAction(action, _client_id, _terminal_id, _) => {
                debug_log!(
                    self,
                    "scrolled panes: {}",
                    self.scrolled_panes
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                if let Some(action) = classify_action(&action) {
                    self.handle_custom_action(action);
                }
                false
            }
            Event::ModeUpdate(m) => {
                debug_log!(self, "mode update: {:?}", m.mode);
                self.handle_mode_switch(m.mode);
                false
            }
            _ => false,
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if let "reload" = pipe_message.name.as_str() {
            let plugin_id = get_plugin_ids().plugin_id;
            debug_log!(self, "reloading plugin id {plugin_id}");
            reload_plugin_with_id(plugin_id);
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("{}", self.status);
    }
}

impl ZmuxScroll {
    fn handle_custom_action(&mut self, action: CustomAction) {
        match action {
            CustomAction::ScrollAt(position) => self.handle_scroll_at(position),
            CustomAction::SwitchMode(mode) => self.handle_mode_switch(mode),
            CustomAction::FocusMayHaveChanged => {
                self.sync_focused_pane();
            }
        }
    }

    fn handle_scroll_at(&mut self, position: Position) {
        // Ensure this works even if the first relevant event after plugin startup is a mouse scroll.
        let Some(tab) = self.active_tab.or_else(|| self.sync_focused_pane()) else {
            return;
        };

        let pane_id = pane_at_position(&self.panes, tab, position.line(), position.column());

        if let Some(pane_id @ PaneId::Terminal(terminal_id)) = pane_id {
            self.scrolled_panes.insert(pane_id);
            self.cur_pane = Some(pane_id);
            self.cur_mode = InputMode::Scroll;
            switch_to_input_mode(&InputMode::Scroll);
            focus_terminal_pane(terminal_id, false, false);
        }
    }

    fn handle_mode_switch(&mut self, mode: InputMode) {
        if mode == InputMode::Tmux {
            return;
        }
        self.cur_mode = mode;

        // Leaving scroll mode while focused on a tracked pane usually means "forget this pane".
        // However, after we restore a pane by calling switch_to_input_mode(Scroll), Zellij can
        // still emit a trailing SwitchToMode(Normal) from the focus/tab movement that got us here.
        // Ignore exactly one such Normal for the restored pane.
        if let Some(pane_id) = self.cur_pane {
            debug_log!(self, "mode switch {:?} - {}", self.cur_mode, pane_id);
            if mode == InputMode::Scroll {
                self.scrolled_panes.insert(pane_id);
            } else {
                self.scrolled_panes.remove(&pane_id);
            }
        }
    }

    fn sync_focused_pane(&mut self) -> Option<usize> {
        let Ok((tab_position, pane_id)) = get_focused_pane_info() else {
            return self.active_tab;
        };
        debug_log!(self, "got new focus: {} - {:?}", tab_position, pane_id);

        let previous_pane = self.cur_pane.replace(pane_id);
        self.active_tab = Some(tab_position);

        if self.cur_mode == InputMode::Scroll {
            if let Some(previous_pane) = previous_pane {
                self.scrolled_panes.insert(previous_pane);
            }
            // When switching to non scrolled pane in scroll mode
            // leave scroll mode
            if !self.scrolled_panes.contains(&pane_id) {
                switch_to_input_mode(&InputMode::Normal);
            }
        } else if self.scrolled_panes.contains(&pane_id) {
            self.cur_mode = InputMode::Scroll;
            switch_to_input_mode(&InputMode::Scroll);
        }

        self.active_tab
    }
}
