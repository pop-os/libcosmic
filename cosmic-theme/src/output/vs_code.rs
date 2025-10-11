use serde::{Deserialize, Serialize};

use crate::Theme;

use super::{OutputError, to_hex};

/// Represents the workbench.colorCustomizations section of a VS Code settings.json file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsTheme {
    #[serde(rename = "editor.background")]
    editor_background: String,
    #[serde(rename = "sideBar.background")]
    sidebar_background: String,
    #[serde(rename = "activityBar.background")]
    activity_bar_background: String,
    #[serde(rename = "notificationCenterHeader.background")]
    notification_center_header_background: String,
    #[serde(rename = "notifications.background")]
    notifications_background: String,
    #[serde(rename = "activityBarTop.activeBackground")]
    activity_bar_top_active_background: String,
    #[serde(rename = "editorGroupHeader.tabsBackground")]
    editor_group_header_tabs_background: String,
    #[serde(rename = "editorGroupHeader.noTabsBackground")]
    editor_group_header_no_tabs_background: String,
    #[serde(rename = "titleBar.activeBackground")]
    title_bar_active_background: String,
    #[serde(rename = "titleBar.inactiveBackground")]
    title_bar_inactive_background: String,
    #[serde(rename = "statusBar.background")]
    status_bar_background: String,
    #[serde(rename = "statusBar.noFolderBackground")]
    status_bar_no_folder_background: String,
    #[serde(rename = "statusBar.debuggingBackground")]
    status_bar_debugging_background: String,
    #[serde(rename = "tab.activeBackground")]
    tab_active_background: String,
    #[serde(rename = "tab.activeBorder")]
    tab_active_border: String,
    #[serde(rename = "tab.activeBorderTop")]
    tab_active_border_top: String,
    #[serde(rename = "tab.hoverBackground")]
    tab_hover_background: String,
    #[serde(rename = "quickInput.background")]
    quick_input_background: String,
    #[serde(rename = "tab.inactiveBackground")]
    tab_inactive_background: String,
    #[serde(rename = "sideBarSectionHeader.background")]
    side_bar_section_header_background: String,
    #[serde(rename = "list.focusOutline")]
    list_focus_outline: String,
    #[serde(rename = "banner.background")]
    banner_background: String,
    #[serde(rename = "breadcrumb.background")]
    breadcrumb_background: String,
    #[serde(rename = "commandCenter.background")]
    command_center_background: String,
    #[serde(rename = "terminal.background")]
    terminal_background: String,
    #[serde(rename = "menu.background")]
    menu_background: String,
    #[serde(rename = "panel.background")]
    panel_background: String,
    #[serde(rename = "peekViewEditorGutter.background")]
    peek_view_editor_gutter_background: String,
    #[serde(rename = "peekViewResult.background")]
    peek_view_result_background: String,
    #[serde(rename = "peekViewTitle.background")]
    peek_view_title_background: String,
    #[serde(rename = "peekViewEditor.background")]
    peek_view_editor_background: String,
    #[serde(rename = "peekViewResult.selectionBackground")]
    peek_view_result_selection_background: String,
    #[serde(rename = "editorWidget.background")]
    editor_widget_background: String,
    #[serde(rename = "editorSuggestWidget.background")]
    editor_suggest_widget_background: String,
    #[serde(rename = "editorHoverWidget.background")]
    editor_hover_widget_background: String,
    #[serde(rename = "input.background")]
    input_background: String,
    #[serde(rename = "dropdown.background")]
    dropdown_background: String,
    #[serde(rename = "settings.checkboxBackground")]
    settings_checkbox_background: String,
    #[serde(rename = "settings.textInputBackground")]
    settings_text_input_background: String,
    #[serde(rename = "settings.numberInputBackground")]
    settings_number_input_background: String,
    #[serde(rename = "settings.dropdownBackground")]
    settings_dropdown_background: String,
    #[serde(rename = "sideBar.dropBackground")]
    side_bar_drop_background: String,
    #[serde(rename = "list.activeSelectionBackground")]
    list_active_selection_background: String,
    #[serde(rename = "list.inactiveSelectionBackground")]
    list_inactive_selection_background: String,
    #[serde(rename = "list.focusBackground")]
    list_focus_background: String,
    #[serde(rename = "list.hoverBackground")]
    list_hover_background: String,

    // text colors
    #[serde(rename = "editor.foreground")]
    editor_foreground: String,
    #[serde(rename = "editorLineNumber.foreground")]
    editor_line_number_foreground: String,
    #[serde(rename = "editorCursor.foreground")]
    editor_cursor_foreground: String,
    #[serde(rename = "sideBar.foreground")]
    side_bar_foreground: String,
    #[serde(rename = "activityBar.foreground")]
    activity_bar_foreground: String,
    #[serde(rename = "statusBar.foreground")]
    status_bar_foreground: String,
    #[serde(rename = "tab.activeForeground")]
    tab_active_foreground: String,
    #[serde(rename = "tab.inactiveForeground")]
    tab_inactive_foreground: String,
    #[serde(rename = "editorGroupHeader.tabsForeground")]
    editor_group_header_tabs_foreground: String,
    #[serde(rename = "sideBarSectionHeader.foreground")]
    side_bar_section_header_foreground: String,
    #[serde(rename = "statusBar.debuggingForeground")]
    status_bar_debugging_foreground: String,
    #[serde(rename = "statusBar.noFolderForeground")]
    status_bar_no_folder_foreground: String,
    #[serde(rename = "editorWidget.foreground")]
    editor_widget_foreground: String,
    #[serde(rename = "editorSuggestWidget.foreground")]
    editor_suggest_widget_foreground: String,
    #[serde(rename = "editorHoverWidget.foreground")]
    editor_hover_widget_foreground: String,
    #[serde(rename = "input.foreground")]
    input_foreground: String,
    #[serde(rename = "dropdown.foreground")]
    dropdown_foreground: String,
    #[serde(rename = "terminal.foreground")]
    terminal_foreground: String,
    #[serde(rename = "menu.foreground")]
    menu_foreground: String,
    #[serde(rename = "panel.foreground")]
    panel_foreground: String,
    #[serde(rename = "peekViewEditorGutter.foreground")]
    peek_view_editor_gutter_foreground: String,
    #[serde(rename = "peekViewResult.selectionForeground")]
    peek_view_result_selection_foreground: String,
    #[serde(rename = "inputOption.activeBorder")]
    input_option_active_border: String,

    // accent colors
    #[serde(rename = "activityBarBadge.background")]
    activity_bar_badge_background: String,
    #[serde(rename = "statusBar.debuggingBorder")]
    status_bar_debugging_border: String,
    #[serde(rename = "button.background")]
    button_background: String,
    #[serde(rename = "button.hoverBackground")]
    button_hover_background: String,
    #[serde(rename = "statusBarItem.remoteBackground")]
    status_bar_item_remote_background: String,

    // accent fg colors
    #[serde(rename = "activityBarBadge.foreground")]
    activity_bar_badge_foreground: String,
    #[serde(rename = "button.foreground")]
    button_foreground: String,
    #[serde(rename = "textLink.foreground")]
    text_link_foreground: String,
    #[serde(rename = "textLink.activeForeground")]
    text_link_active_foreground: String,
    #[serde(rename = "peekView.border")]
    peek_view_border: String,
    #[serde(rename = "settings.checkboxForeground")]
    settings_checkbox_foreground: String,
}

impl From<Theme> for VsTheme {
    fn from(theme: Theme) -> Self {
        Self {
            editor_background: format!("#{}", to_hex(theme.background.base)),
            sidebar_background: format!("#{}", to_hex(theme.primary.base)),
            activity_bar_background: format!("#{}", to_hex(theme.primary.base)),
            notification_center_header_background: format!("#{}", to_hex(theme.background.base)),
            notifications_background: format!("#{}", to_hex(theme.background.base)),
            activity_bar_top_active_background: format!("#{}", to_hex(theme.primary.base)),
            editor_group_header_tabs_background: format!("#{}", to_hex(theme.background.base)),
            editor_group_header_no_tabs_background: format!("#{}", to_hex(theme.background.base)),
            title_bar_active_background: format!("#{}", to_hex(theme.background.component.base)),
            title_bar_inactive_background: format!(
                "#{}",
                to_hex(theme.background.component.disabled)
            ),
            status_bar_background: format!("#{}", to_hex(theme.background.base)),
            status_bar_no_folder_background: format!("#{}", to_hex(theme.background.base)),
            status_bar_debugging_background: format!("#{}", to_hex(theme.background.base)),
            tab_active_background: format!("#{}", to_hex(theme.primary.component.pressed)),
            tab_active_border: format!("#{}", to_hex(theme.accent.base)),
            tab_active_border_top: format!("#{}", to_hex(theme.accent.base)),
            tab_hover_background: format!("#{}", to_hex(theme.primary.component.hover)),
            tab_inactive_background: format!("#{}", to_hex(theme.primary.component.base)),
            quick_input_background: format!("#{}", to_hex(theme.primary.base)),
            side_bar_section_header_background: format!("#{}", to_hex(theme.primary.base)),
            banner_background: format!("#{}", to_hex(theme.primary.base)),
            breadcrumb_background: format!("#{}", to_hex(theme.primary.base)),
            command_center_background: format!("#{}", to_hex(theme.primary.base)),
            terminal_background: format!("#{}", to_hex(theme.primary.base)),
            menu_background: format!("#{}", to_hex(theme.primary.base)),
            panel_background: format!("#{}", to_hex(theme.primary.base)),
            peek_view_editor_gutter_background: format!("#{}", to_hex(theme.background.base)),
            peek_view_result_background: format!("#{}", to_hex(theme.background.base)),
            peek_view_title_background: format!("#{}", to_hex(theme.background.base)),
            peek_view_editor_background: format!("#{}", to_hex(theme.background.base)),
            peek_view_result_selection_background: format!("#{}", to_hex(theme.background.base)),
            editor_widget_background: format!("#{}", to_hex(theme.background.base)),
            editor_suggest_widget_background: format!("#{}", to_hex(theme.background.base)),
            editor_hover_widget_background: format!("#{}", to_hex(theme.background.base)),
            input_background: format!("#{}", to_hex(theme.background.base)),
            dropdown_background: format!("#{}", to_hex(theme.background.base)),
            settings_checkbox_background: format!("#{}", to_hex(theme.background.base)),
            settings_text_input_background: format!("#{}", to_hex(theme.background.base)),
            settings_number_input_background: format!("#{}", to_hex(theme.background.base)),
            settings_dropdown_background: format!("#{}", to_hex(theme.background.base)),
            side_bar_drop_background: format!("#{}", to_hex(theme.background.base)),
            list_active_selection_background: format!("#{}", to_hex(theme.primary.base)),
            list_inactive_selection_background: format!("#{}", to_hex(theme.primary.base)),
            list_focus_background: format!("#{}", to_hex(theme.primary.base)),
            list_hover_background: format!("#{}", to_hex(theme.primary.base)),
            editor_foreground: format!("#{}", to_hex(theme.background.on)),
            editor_line_number_foreground: format!("#{}", to_hex(theme.background.on)),
            editor_cursor_foreground: format!("#{}", to_hex(theme.background.on)),
            side_bar_foreground: format!("#{}", to_hex(theme.primary.on)),
            activity_bar_foreground: format!("#{}", to_hex(theme.primary.on)),
            status_bar_foreground: format!("#{}", to_hex(theme.primary.on)),
            tab_active_foreground: format!("#{}", to_hex(theme.primary.on)),
            tab_inactive_foreground: format!("#{}", to_hex(theme.primary.on)),
            editor_group_header_tabs_foreground: format!("#{}", to_hex(theme.primary.on)),
            side_bar_section_header_foreground: format!("#{}", to_hex(theme.primary.on)),
            status_bar_debugging_foreground: format!("#{}", to_hex(theme.primary.on)),
            status_bar_no_folder_foreground: format!("#{}", to_hex(theme.primary.on)),
            editor_widget_foreground: format!("#{}", to_hex(theme.primary.on)),
            editor_suggest_widget_foreground: format!("#{}", to_hex(theme.primary.on)),
            editor_hover_widget_foreground: format!("#{}", to_hex(theme.primary.on)),
            input_foreground: format!("#{}", to_hex(theme.primary.on)),
            dropdown_foreground: format!("#{}", to_hex(theme.primary.on)),
            terminal_foreground: format!("#{}", to_hex(theme.primary.on)),
            menu_foreground: format!("#{}", to_hex(theme.primary.on)),
            panel_foreground: format!("#{}", to_hex(theme.primary.on)),
            peek_view_editor_gutter_foreground: format!("#{}", to_hex(theme.primary.on)),
            peek_view_result_selection_foreground: format!("#{}", to_hex(theme.primary.on)),
            input_option_active_border: format!("#{}", to_hex(theme.accent.base)),
            activity_bar_badge_background: format!("#{}", to_hex(theme.accent.base)),
            activity_bar_badge_foreground: format!("#{}", to_hex(theme.accent.on)),
            status_bar_debugging_border: format!("#{}", to_hex(theme.accent.base)),
            list_focus_outline: format!("#{}", to_hex(theme.accent.base)),
            button_background: format!("#{}", to_hex(theme.accent_button.base)),
            button_hover_background: format!("#{}", to_hex(theme.accent_button.hover)),
            status_bar_item_remote_background: format!("#{}", to_hex(theme.accent.base)),
            button_foreground: format!("#{}", to_hex(theme.accent_button.on)),
            text_link_foreground: format!("#{}", to_hex(theme.accent.base)),
            text_link_active_foreground: format!("#{}", to_hex(theme.accent.base)),
            peek_view_border: format!("#{}", to_hex(theme.accent.base)),
            settings_checkbox_foreground: format!("#{}", to_hex(theme.accent.base)),
        }
    }
}

impl Theme {
    #[cold]
    pub fn apply_vs_code(self) -> Result<(), OutputError> {
        let vs_theme = VsTheme::from(self);
        let mut config_dir = dirs::config_dir().ok_or(OutputError::MissingConfigDir)?;
        config_dir.extend(["Code", "User"]);
        let vs_code_dir = config_dir;
        if !vs_code_dir.exists() {
            std::fs::create_dir_all(&vs_code_dir).map_err(OutputError::Io)?;
        }

        // just add the json entry for workbench.colorCustomizations
        let settings_file = vs_code_dir.join("settings.json");
        let settings = std::fs::read_to_string(&settings_file).unwrap_or_default();
        let mut settings: serde_json::Value = serde_json::from_str(&settings)?;
        settings["workbench.colorCustomizations"] = serde_json::to_value(vs_theme).unwrap();
        settings["window.autoDetectColorScheme"] = serde_json::Value::Bool(true);
        std::fs::write(
            &settings_file,
            serde_json::to_string_pretty(&settings).unwrap(),
        )
        .map_err(OutputError::Io)?;

        Ok(())
    }

    #[cold]
    pub fn reset_vs_code() -> Result<(), OutputError> {
        let mut config_dir = dirs::config_dir().ok_or(OutputError::MissingConfigDir)?;
        config_dir.extend(["Code", "User", "settings.json"]);
        let settings_file = config_dir;
        // just remove the json entry for workbench.colorCustomizations
        let settings = std::fs::read_to_string(&settings_file).unwrap_or_default();
        let mut settings: serde_json::Value = serde_json::from_str(&settings).unwrap_or_default();
        settings["workbench.colorCustomizations"] = serde_json::Value::Null;

        std::fs::write(
            &settings_file,
            serde_json::to_string_pretty(&settings).unwrap(),
        )
        .map_err(OutputError::Io)?;

        Ok(())
    }
}
