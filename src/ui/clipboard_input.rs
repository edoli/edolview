use egui::{Event, Key, Modifiers};

use crate::res::{COPY_SHORTCUTS, CUT_SHORTCUTS, PASTE_SHORTCUTS};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ClipboardAction {
    Copy,
    Cut,
    Paste,
}

pub(super) fn clipboard_action(key: Key, modifiers: Modifiers) -> Option<ClipboardAction> {
    for (shortcuts, action) in [
        (COPY_SHORTCUTS, ClipboardAction::Copy),
        (CUT_SHORTCUTS, ClipboardAction::Cut),
        (PASTE_SHORTCUTS, ClipboardAction::Paste),
    ] {
        if shortcuts.iter().any(|shortcut| {
            key == shortcut.logical_key
                && if shortcut.modifiers.command {
                    modifiers.matches_exact(shortcut.modifiers)
                } else {
                    // Like egui-winit, dedicated keys ignore modifiers, and
                    // Windows aliases only require their listed modifier.
                    modifiers.contains(shortcut.modifiers)
                }
        }) {
            return Some(action);
        }
    }
    None
}

/// Replace egui-winit's automatic clipboard shortcuts without reading images on
/// the UI thread. Unfocused paste keys are left for the background image loader.
pub(super) fn translate_text_clipboard_shortcuts(
    events: &mut Vec<Event>,
    text_edit_focused: bool,
    mut read_text: impl FnMut() -> Result<String, arboard::Error>,
) -> Option<arboard::Error> {
    let mut failure = None;
    events.retain_mut(|event| {
        let Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } = event
        else {
            return true;
        };
        match clipboard_action(*key, *modifiers) {
            Some(ClipboardAction::Copy) => *event = Event::Copy,
            Some(ClipboardAction::Cut) => *event = Event::Cut,
            Some(ClipboardAction::Paste) if text_edit_focused => match read_text() {
                Ok(text) if !text.is_empty() => *event = Event::Paste(text.replace("\r\n", "\n")),
                Ok(_) | Err(arboard::Error::ContentNotAvailable) => return false,
                Err(error) => {
                    failure = Some(error);
                    return false;
                }
            },
            _ => {}
        }
        true
    });
    failure
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, OutputCommand, RawInput};

    fn press(key: Key) -> Vec<Event> {
        [true, false]
            .into_iter()
            .map(|pressed| Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: if cfg!(target_os = "macos") {
                    Modifiers {
                        command: true,
                        mac_cmd: true,
                        ..Modifiers::NONE
                    }
                } else {
                    Modifiers {
                        command: true,
                        ctrl: true,
                        ..Modifiers::NONE
                    }
                },
            })
            .collect()
    }

    #[test]
    fn image_paste_does_not_read_the_clipboard_on_the_ui_thread() {
        let mut events = press(Key::V);
        let original = events.clone();
        assert!(
            translate_text_clipboard_shortcuts(&mut events, false, || panic!("Image paste belongs to the worker"))
                .is_none()
        );
        assert_eq!(events, original);
    }

    #[test]
    fn text_editor_paste_copy_and_cut_work_with_native_key_events() {
        let ctx = Context::default();
        let mut text = "old text".to_string();
        let mut frame = |events: Vec<Event>, focus: bool| {
            let mut output = ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let focused = ctx.text_edit_focused();
                    ctx.input_mut(|input| {
                        assert!(translate_text_clipboard_shortcuts(&mut input.events, focused, || Ok(
                            "한글🙂\r\nnew text".into()
                        ))
                        .is_none());
                    });
                    let response = ui.add(egui::TextEdit::multiline(&mut text).id_salt("clipboard-test"));
                    if focus {
                        response.request_focus();
                    }
                },
            );
            output.textures_delta.clear();
            (text.clone(), output.platform_output.commands)
        };
        frame(Vec::new(), true);
        let events = press(Key::A).into_iter().chain(press(Key::V)).collect();
        assert_eq!(frame(events, false).0, "한글🙂\nnew text");
        let events = press(Key::A).into_iter().chain(press(Key::C)).collect();
        let (_, commands) = frame(events, false);
        assert!(commands
            .iter()
            .any(|command| matches!(command, OutputCommand::CopyText(value) if value == "한글🙂\nnew text")));
        let (text, commands) = frame(press(Key::X), false);
        assert!(text.is_empty());
        assert!(commands
            .iter()
            .any(|command| matches!(command, OutputCommand::CopyText(value) if value == "한글🙂\nnew text")));
    }

    #[test]
    fn empty_or_image_only_clipboard_does_not_delete_selected_text() {
        for result in [Ok(String::new()), Err(arboard::Error::ContentNotAvailable)] {
            let mut events = press(Key::V);
            let mut result = Some(result);
            assert!(translate_text_clipboard_shortcuts(&mut events, true, || result.take().unwrap()).is_none());
            assert!(!events.iter().any(|event| matches!(event, Event::Paste(_))));
        }
        let mut events = press(Key::V);
        assert!(matches!(
            translate_text_clipboard_shortcuts(&mut events, true, || Err(arboard::Error::ClipboardOccupied)),
            Some(arboard::Error::ClipboardOccupied)
        ));
    }

    #[test]
    fn translated_numeric_paste_uses_the_existing_field_filter() {
        let mut events = press(Key::V);
        translate_text_clipboard_shortcuts(&mut events, true, || Ok("1한글2🙂 + pi".into()));
        super::super::field_input::filter_field_events(&mut events);
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::Paste(text) if text == "12 + pi")));
    }

    #[cfg(windows)]
    #[test]
    fn windows_clipboard_aliases_are_preserved() {
        assert!(clipboard_action(Key::Insert, Modifiers::CTRL) == Some(ClipboardAction::Copy));
        assert!(clipboard_action(Key::Delete, Modifiers::SHIFT) == Some(ClipboardAction::Cut));
        assert!(clipboard_action(Key::Insert, Modifiers::SHIFT) == Some(ClipboardAction::Paste));
        assert!(clipboard_action(Key::Insert, Modifiers::CTRL.plus(Modifiers::SHIFT)) == Some(ClipboardAction::Copy));
        assert!(clipboard_action(Key::Delete, Modifiers::CTRL.plus(Modifiers::SHIFT)) == Some(ClipboardAction::Cut));
    }

    #[test]
    fn dedicated_clipboard_keys_preserve_native_modifier_behavior() {
        let modifiers = Modifiers::CTRL.plus(Modifiers::SHIFT).plus(Modifiers::ALT);
        for (key, action) in [
            (Key::Copy, ClipboardAction::Copy),
            (Key::Cut, ClipboardAction::Cut),
            (Key::Paste, ClipboardAction::Paste),
        ] {
            assert!(clipboard_action(key, modifiers) == Some(action));
        }
        for key in [Key::C, Key::X, Key::V] {
            assert!(clipboard_action(key, Modifiers::COMMAND.plus(Modifiers::ALT)).is_none());
        }
    }
}
