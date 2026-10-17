use eframe::egui::{Event, ImeEvent, RawInput};

fn allowed_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            ' ' | '\t' | '\r' | '\n' | '.' | ',' | ':' | '+' | '-' | '*' | '/' | '%' | '^' | '(' | ')' | '[' | ']'
        )
}

pub(super) fn filter_field_input(input: &mut RawInput) {
    // All current in-app editors contain numeric expressions or rectangles.
    // Filter before egui so this also covers DragValue's internal text editor.
    // File paths, displayed labels and keyboard/mouse events are unaffected.
    input.events.retain_mut(|event| match event {
        Event::Text(text) | Event::Paste(text) => {
            text.retain(allowed_character);
            // Ignore rejected input entirely so it cannot delete a selection.
            !text.is_empty()
        }
        Event::Ime(ImeEvent::Commit(text)) => {
            text.retain(allowed_character);
            if text.is_empty() {
                false
            } else {
                *event = Event::Text(std::mem::take(text));
                true
            }
        }
        // Do not place composition text or IME cursor edits in ASCII-only fields.
        Event::Ime(_) => false,
        _ => true,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{DroppedFile, Key, Modifiers};

    #[derive(Debug)]
    struct TestDroppedFile;

    impl DroppedFile for TestDroppedFile {
        fn path(&self) -> &std::path::Path {
            std::path::Path::new("한글🙂.png")
        }

        fn bytes(&self) -> Result<Vec<u8>, String> {
            panic!("Input filtering must not read dropped files")
        }
    }

    #[test]
    fn keeps_expression_and_rectangle_input() {
        let text = "AbcXYZ0123456789 1.25e-3 + pi * (2 ^ 3) / 4 % 2, [-10:20, 0:30]";
        let mut input = RawInput {
            events: vec![Event::Text(text.to_owned()), Event::Paste(text.to_owned())],
            ..Default::default()
        };
        filter_field_input(&mut input);
        assert!(
            matches!(&input.events[..], [Event::Text(typed), Event::Paste(pasted)] if typed == text && pasted == text)
        );
    }

    #[test]
    fn filters_typing_and_pasting_before_the_editor_sees_them() {
        let mut input = RawInput {
            events: vec![
                Event::Text("한글🙂".to_owned()),
                Event::Paste("한글👨‍👩‍👦π１２".to_owned()),
                Event::Text("1한글2🙂 + pi".to_owned()),
                Event::Paste("[0:10, 0:20] 한글🙂!@#$&={}".to_owned()),
            ],
            ..Default::default()
        };
        filter_field_input(&mut input);
        assert!(
            matches!(&input.events[..], [Event::Text(typed), Event::Paste(pasted)] if typed == "12 + pi" && pasted == "[0:10, 0:20] ")
        );
    }

    #[test]
    fn rejects_ime_composition_and_keeps_only_ascii_commits() {
        let mut input = RawInput {
            events: vec![
                Event::Ime(ImeEvent::Preedit {
                    text: "ㅎ".to_owned(),
                    active_range_chars: Some(0..1),
                }),
                Event::Ime(ImeEvent::Commit("한🙂".to_owned())),
                Event::Ime(ImeEvent::DeleteSurrounding {
                    before_chars: 1,
                    after_chars: 0,
                }),
                Event::Ime(ImeEvent::Commit("1 + pi".to_owned())),
            ],
            ..Default::default()
        };
        filter_field_input(&mut input);
        assert!(matches!(&input.events[..], [Event::Text(text)] if text == "1 + pi"));
    }

    #[test]
    fn preserves_editing_keys_shortcuts_and_unicode_file_names() {
        let mut input = RawInput {
            events: vec![
                Event::Key {
                    key: Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
                Event::Key {
                    key: Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::CTRL,
                },
                Event::Copy,
                Event::Cut,
            ],
            dropped_files: vec![std::sync::Arc::new(TestDroppedFile)],
            ..Default::default()
        };
        let events = input.events.clone();
        filter_field_input(&mut input);
        assert_eq!(input.events, events);
        assert_eq!(input.dropped_files[0].path(), std::path::Path::new("한글🙂.png"));
    }

    #[test]
    fn rejected_input_preserves_a_selected_drag_value_and_allows_the_next_edit() {
        let ctx = eframe::egui::Context::default();
        let mut value = 12.0;
        let mut edit = |events, request_focus| {
            let mut input = RawInput {
                events,
                ..Default::default()
            };
            filter_field_input(&mut input);
            let mut output = ctx.run_ui(input, |ui| {
                let response = ui.add(
                    eframe::egui::DragValue::new(&mut value)
                        .custom_parser(crate::util::expression::parse_number_expression),
                );
                if request_focus {
                    response.request_focus();
                }
            });
            output.textures_delta.clear();
            value
        };
        edit(Vec::new(), true);
        edit(
            vec![Event::Key {
                key: Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
            false,
        );
        assert_eq!(
            edit(
                vec![
                    Event::Text("한글🙂".to_owned()),
                    Event::Paste("한글🙂".to_owned()),
                    Event::Ime(ImeEvent::Preedit {
                        text: "ㅎ".to_owned(),
                        active_range_chars: Some(0..1)
                    }),
                    Event::Ime(ImeEvent::Commit("한".to_owned())),
                ],
                false,
            ),
            12.0,
        );
        let edited_value = edit(vec![Event::Paste("2 * pi".to_owned())], false);
        assert!(
            (edited_value - 2.0 * std::f64::consts::PI).abs() < 1e-12,
            "edited value: {edited_value}"
        );
    }
}
