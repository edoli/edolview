/// Column widths for color, coordinates, angle, a spacer, and image details.
pub(super) fn column_rows(available: f32, spacing: f32, minimum: [f32; 5]) -> (f32, Vec<[f32; 5]>) {
    let preferred = [400.0_f32, 160.0, 40.0, 0.0, 108.0];
    let minimum_total: f32 = minimum.iter().sum();
    // Remove only unused space; never squeeze a column below its current text.
    let spacing = if minimum_total <= available {
        spacing.min((available - minimum_total) / 4.0)
    } else {
        spacing.min(available / 4.0)
    };
    let budget = (available - spacing * 4.0).max(0.0);
    let fit = |indices: &[usize]| {
        let mut widths = [0.0; 5];
        let natural: f32 = indices.iter().map(|&i| minimum[i]).sum();
        let slack: f32 = indices.iter().map(|&i| (preferred[i] - minimum[i]).max(0.0)).sum();
        let fraction = if slack > 0.0 {
            ((budget - natural) / slack).clamp(0.0, 1.0)
        } else {
            0.0
        };
        for &i in indices {
            widths[i] = (minimum[i] + (preferred[i] - minimum[i]).max(0.0) * fraction).min(budget);
        }
        widths[3] = (budget - widths.iter().sum::<f32>()).max(0.0);
        widths
    };
    if minimum_total <= budget {
        return (spacing, vec![fit(&[0, 1, 2, 4])]);
    }

    // Very long values need another row after all horizontal padding is gone.
    let mut rows = Vec::new();
    for pair in [[0, 1], [2, 4]] {
        if pair.iter().map(|&i| minimum[i]).sum::<f32>() <= budget {
            rows.push(fit(&pair));
        } else {
            // Keep the coordinate editor in the first row so resizing does not
            // change its egui ID and interrupt an in-progress edit.
            let order = if pair == [0, 1] { [1, 0] } else { pair };
            rows.extend(order.map(|i| fit(&[i])));
        }
    }
    (spacing, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_layout_preserves_existing_positions_and_narrow_layout_fits_content() {
        let minimum = [120.0, 130.0, 40.0, 0.0, 105.0];
        let (gap, rows) = column_rows(1280.0, 8.0, minimum);
        assert_eq!(gap, 8.0);
        assert_eq!(rows, vec![[400.0, 160.0, 40.0, 540.0, 108.0]]);
        for width in [740.0, 739.0, 720.0, 500.0, 427.0, 410.0, 395.0] {
            let (gap, rows) = column_rows(width, 8.0, minimum);
            assert_eq!(rows.len(), 1);
            assert!((rows[0].iter().sum::<f32>() + gap * 4.0 - width).abs() < 0.001);
            for i in [0, 1, 2, 4] {
                assert!(rows[0][i] >= minimum[i]);
            }
        }
    }

    #[test]
    fn long_values_get_space_and_wrap_groups_only_when_they_cannot_fit_together() {
        let minimum = [360.0, 240.0, 48.0, 0.0, 160.0];
        for width in [1280.0, 840.0, 808.0, 807.0, 720.0, 500.0] {
            let (gap, rows) = column_rows(width, 8.0, minimum);
            assert_eq!(rows.len() > 1, width < minimum.iter().sum());
            for row in &rows {
                assert!((row.iter().sum::<f32>() + gap * 4.0 - width).abs() < 0.001);
            }
            for i in [0, 1, 2, 4] {
                assert_eq!(rows.iter().filter(|row| row[i] >= minimum[i]).count(), 1);
            }
        }
    }
}
