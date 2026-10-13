// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later

//! Customize toolbar: which buttons the toolbar draws, and in what order.
//! Reachable from View > Customize toolbar.
//!
//! The rows are every button the toolbar can draw, in the order it lays them
//! out, hidden ones included. Edits apply to the toolbar as they are made —
//! the dialog is a view onto the setting, not a form to submit.

use crate::app::{App, El, Message, WinKind};
use crate::model::{self, ToolbarItem, ToolbarMove, TOOLBAR_MAX_VISIBLE};
use crate::windows::{check, dlg_btn_auto, dlg_btn_primary};
use crate::{i18n::tr, theme};
use iced::widget::{button, column, container, row, text};
use iced::Length;

/// One move button, disabled when the row is already at that end.
fn mover<'a>(label: String, item: ToolbarItem, to: ToolbarMove, enabled: bool) -> El<'a> {
    let mut b = button(text(label).size(theme::FONT_SIZE))
        .padding([2, 8])
        .style(theme::btn);
    if enabled {
        b = b.on_press(Message::ToolbarMove(item, to));
    }
    b.into()
}

pub fn view(app: &App) -> El<'_> {
    let prefs = &app.cfg.settings.toolbar;
    let shown = model::toolbar_visible_count(prefs);
    let full = shown >= TOOLBAR_MAX_VISIBLE;
    let mut list = column![].spacing(6);
    for (i, pref) in prefs.iter().enumerate() {
        let (first, last) = (i == 0, i + 1 == prefs.len());
        let toggle = check(pref.visible, tr(pref.id.label()));
        let toggle = if pref.visible || !full {
            toggle.on_toggle(move |_| Message::ToolbarToggle(pref.id))
        } else {
            toggle
        };
        list = list.push(
            row![
                container(toggle).width(Length::Fill),
                mover(tr("First"), pref.id, ToolbarMove::First, !first),
                mover("\u{25b2}".into(), pref.id, ToolbarMove::Earlier, !first),
                mover("\u{25bc}".into(), pref.id, ToolbarMove::Later, !last),
                mover(tr("Last"), pref.id, ToolbarMove::Last, !last),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        );
    }

    let count = tr("Toolbar buttons shown: {shown} of {max}")
        .replace("{shown}", &shown.to_string())
        .replace("{max}", &TOOLBAR_MAX_VISIBLE.to_string());
    container(
        column![
            text(count)
                .size(theme::FONT_SIZE - 1.0)
                .color(theme::dim_text(&iced::Theme::Light)),
            crate::ui::scroll(list).height(Length::Fill),
            row![
                dlg_btn_auto(tr("Reset"), Some(Message::ToolbarReset)),
                iced::widget::space::horizontal(),
                dlg_btn_primary(
                    tr("OK"),
                    app.win_of(WinKind::Toolbar).map(Message::CloseThis)
                ),
            ]
            .align_y(iced::Alignment::Center),
        ]
        .spacing(10)
        .padding(16),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::window)
    .into()
}
