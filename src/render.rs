use std::collections::BTreeMap;

const BLOCK_CHAR: char = '\u{2588}'; // Unicode full block

pub fn render_histogram(year_counts: &BTreeMap<i32, usize>, width: usize) -> Vec<String> {
    if year_counts.is_empty() {
        return vec![];
    }

    let max_count = *year_counts.values().max().unwrap_or(&0);
    if max_count == 0 {
        // All zeros, just format with no bars
        return year_counts
            .iter()
            .map(|(year, count)| format!("{}  {}", year, count))
            .collect();
    }

    year_counts
        .iter()
        .map(|(year, &count)| {
            let bar = if count == 0 {
                String::new()
            } else {
                // Scale to width, guarantee at least 1 block for non-zero
                let scaled = ((count as f64 / max_count as f64) * width as f64).round() as usize;
                let bar_width = scaled.max(1);
                BLOCK_CHAR.to_string().repeat(bar_width)
            };
            format!("{} {} {}", year, bar, count)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_histogram_empty() {
        let hist = BTreeMap::new();
        let lines = render_histogram(&hist, 50);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_render_histogram_single_year() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 10);
        let lines = render_histogram(&hist, 50);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("2020 "));
        assert!(lines[0].ends_with(" 10"));
        assert!(lines[0].contains(BLOCK_CHAR));
    }

    #[test]
    fn test_render_histogram_with_zero_count() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 10);
        hist.insert(2021, 0);
        hist.insert(2022, 5);
        let lines = render_histogram(&hist, 50);
        assert_eq!(lines.len(), 3);

        // Year 2021 should have no blocks
        let line_2021 = &lines[1];
        assert!(line_2021.starts_with("2021 "));
        assert!(line_2021.ends_with(" 0"));
        // Should have format "2021  0" (two spaces, no bar)
        assert_eq!(line_2021, "2021  0");
    }

    #[test]
    fn test_render_histogram_scaling() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 100); // max
        hist.insert(2021, 50); // half
        hist.insert(2022, 1); // minimum non-zero

        let lines = render_histogram(&hist, 50);

        // 2020 should have 50 blocks (100%)
        let blocks_2020 = lines[0].matches(BLOCK_CHAR).count();
        assert_eq!(blocks_2020, 50);

        // 2021 should have ~25 blocks (50%)
        let blocks_2021 = lines[1].matches(BLOCK_CHAR).count();
        assert_eq!(blocks_2021, 25);

        // 2022 should have at least 1 block (guaranteed minimum)
        let blocks_2022 = lines[2].matches(BLOCK_CHAR).count();
        assert_eq!(blocks_2022, 1);
    }

    #[test]
    fn test_render_histogram_width_1() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 100);
        hist.insert(2021, 50);

        let lines = render_histogram(&hist, 1);

        // Both should have exactly 1 block (width=1, both non-zero)
        let blocks_2020 = lines[0].matches(BLOCK_CHAR).count();
        let blocks_2021 = lines[1].matches(BLOCK_CHAR).count();
        assert_eq!(blocks_2020, 1);
        assert_eq!(blocks_2021, 1);
    }

    #[test]
    fn test_render_histogram_large_width() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 10);

        let lines = render_histogram(&hist, 200);

        // Should have 200 blocks for max value at width 200
        let blocks_2020 = lines[0].matches(BLOCK_CHAR).count();
        assert_eq!(blocks_2020, 200);
    }

    #[test]
    fn test_render_histogram_all_zeros() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 0);
        hist.insert(2021, 0);

        let lines = render_histogram(&hist, 50);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "2020  0");
        assert_eq!(lines[1], "2021  0");
    }

    #[test]
    fn test_render_histogram_format() {
        let mut hist = BTreeMap::new();
        hist.insert(2020, 5);

        let lines = render_histogram(&hist, 10);
        assert_eq!(lines.len(), 1);

        // Format should be "YEAR BAR COUNT"
        let line = &lines[0];
        assert!(line.starts_with("2020 "));
        assert!(line.ends_with(" 5"));

        // Should contain blocks
        assert!(line.contains(BLOCK_CHAR));
    }
}
