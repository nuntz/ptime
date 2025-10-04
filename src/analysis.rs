use crate::metadata::PhotoMeta;
use chrono::Datelike;
use std::collections::BTreeMap;

pub fn find_oldest(photos: &[PhotoMeta]) -> Option<&PhotoMeta> {
    if photos.is_empty() {
        return None;
    }

    photos.iter().min_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    })
}

pub fn find_latest(photos: &[PhotoMeta]) -> Option<&PhotoMeta> {
    if photos.is_empty() {
        return None;
    }

    photos.iter().max_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| b.rel_path.cmp(&a.rel_path)) // reverse for lexicographic
    })
}

pub fn build_histogram(photos: &[PhotoMeta]) -> BTreeMap<i32, usize> {
    if photos.is_empty() {
        return BTreeMap::new();
    }

    // Count photos per year
    let mut year_counts = BTreeMap::new();
    for photo in photos {
        *year_counts.entry(photo.date.year()).or_insert(0) += 1;
    }

    // Find min and max years
    let min_year = *year_counts.keys().min().unwrap();
    let max_year = *year_counts.keys().max().unwrap();

    // Fill in all years between min and max with zero counts
    let mut complete_histogram = BTreeMap::new();
    for year in min_year..=max_year {
        complete_histogram.insert(year, *year_counts.get(&year).unwrap_or(&0));
    }

    complete_histogram
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn make_photo(path: &str, year: i32, month: u32, day: u32) -> PhotoMeta {
        PhotoMeta {
            rel_path: PathBuf::from(path),
            date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
        }
    }

    #[test]
    fn test_find_oldest_empty() {
        let photos: Vec<PhotoMeta> = vec![];
        assert!(find_oldest(&photos).is_none());
    }

    #[test]
    fn test_find_oldest_single() {
        let photos = vec![make_photo("a.jpg", 2020, 1, 1)];
        let result = find_oldest(&photos).unwrap();
        assert_eq!(result.rel_path, PathBuf::from("a.jpg"));
    }

    #[test]
    fn test_find_oldest_multiple() {
        let photos = vec![
            make_photo("c.jpg", 2022, 6, 15),
            make_photo("a.jpg", 2020, 3, 10),
            make_photo("b.jpg", 2021, 12, 25),
        ];
        let result = find_oldest(&photos).unwrap();
        assert_eq!(result.rel_path, PathBuf::from("a.jpg"));
        assert_eq!(result.date.year(), 2020);
    }

    #[test]
    fn test_find_oldest_with_tie_breaking() {
        let photos = vec![
            make_photo("z.jpg", 2020, 1, 1),
            make_photo("a.jpg", 2020, 1, 1),
            make_photo("m.jpg", 2020, 1, 1),
        ];
        let result = find_oldest(&photos).unwrap();
        assert_eq!(result.rel_path, PathBuf::from("a.jpg"));
    }

    #[test]
    fn test_find_latest_empty() {
        let photos: Vec<PhotoMeta> = vec![];
        assert!(find_latest(&photos).is_none());
    }

    #[test]
    fn test_find_latest_single() {
        let photos = vec![make_photo("a.jpg", 2020, 1, 1)];
        let result = find_latest(&photos).unwrap();
        assert_eq!(result.rel_path, PathBuf::from("a.jpg"));
    }

    #[test]
    fn test_find_latest_multiple() {
        let photos = vec![
            make_photo("c.jpg", 2022, 6, 15),
            make_photo("a.jpg", 2020, 3, 10),
            make_photo("b.jpg", 2021, 12, 25),
        ];
        let result = find_latest(&photos).unwrap();
        assert_eq!(result.rel_path, PathBuf::from("c.jpg"));
        assert_eq!(result.date.year(), 2022);
    }

    #[test]
    fn test_find_latest_with_tie_breaking() {
        let photos = vec![
            make_photo("z.jpg", 2023, 12, 31),
            make_photo("a.jpg", 2023, 12, 31),
            make_photo("m.jpg", 2023, 12, 31),
        ];
        let result = find_latest(&photos).unwrap();
        assert_eq!(result.rel_path, PathBuf::from("a.jpg"));
    }

    #[test]
    fn test_build_histogram_empty() {
        let photos: Vec<PhotoMeta> = vec![];
        let hist = build_histogram(&photos);
        assert!(hist.is_empty());
    }

    #[test]
    fn test_build_histogram_single_year() {
        let photos = vec![
            make_photo("a.jpg", 2020, 1, 1),
            make_photo("b.jpg", 2020, 6, 15),
            make_photo("c.jpg", 2020, 12, 31),
        ];
        let hist = build_histogram(&photos);
        assert_eq!(hist.len(), 1);
        assert_eq!(hist.get(&2020), Some(&3));
    }

    #[test]
    fn test_build_histogram_multiple_years() {
        let photos = vec![
            make_photo("a.jpg", 2020, 1, 1),
            make_photo("b.jpg", 2020, 6, 15),
            make_photo("c.jpg", 2022, 3, 10),
            make_photo("d.jpg", 2023, 7, 20),
        ];
        let hist = build_histogram(&photos);
        assert_eq!(hist.len(), 4); // 2020, 2021 (gap), 2022, 2023
        assert_eq!(hist.get(&2020), Some(&2));
        assert_eq!(hist.get(&2021), Some(&0)); // Gap filled
        assert_eq!(hist.get(&2022), Some(&1));
        assert_eq!(hist.get(&2023), Some(&1));
    }

    #[test]
    fn test_build_histogram_fills_gaps() {
        let photos = vec![
            make_photo("a.jpg", 2018, 1, 1),
            make_photo("b.jpg", 2022, 1, 1),
        ];
        let hist = build_histogram(&photos);
        assert_eq!(hist.len(), 5); // 2018, 2019, 2020, 2021, 2022
        assert_eq!(hist.get(&2018), Some(&1));
        assert_eq!(hist.get(&2019), Some(&0));
        assert_eq!(hist.get(&2020), Some(&0));
        assert_eq!(hist.get(&2021), Some(&0));
        assert_eq!(hist.get(&2022), Some(&1));
    }
}
