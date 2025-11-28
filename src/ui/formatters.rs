//! Formatting utilities for displaying values in the UI

use chrono::Utc;
use timeago::Formatter;

const KB: f64 = 1024.0;
const MB: f64 = KB * 1024.0;
const GB: f64 = MB * 1024.0;

/// Formats a byte value with the appropriate unit
fn format_byte_value(
    value: f64,
    suffix: &str,
    include_b: bool,
    precisions: (usize, usize, usize, usize),
) -> String {
    let (gb_prec, mb_prec, kb_prec, b_prec) = precisions;
    let b = if include_b { "B" } else { "" };

    if value >= GB {
        format!("{:.prec$} G{}{}", value / GB, b, suffix, prec = gb_prec)
    } else if value >= MB {
        format!("{:.prec$} M{}{}", value / MB, b, suffix, prec = mb_prec)
    } else if value >= KB {
        format!("{:.prec$} K{}{}", value / KB, b, suffix, prec = kb_prec)
    } else {
        format!("{:.prec$} B{}", value, suffix, prec = b_prec)
    }
}

/// Formats bytes into a human-readable string (B, K, M, G)
pub fn format_bytes(bytes: u64) -> String {
    format_byte_value(bytes as f64, "", false, (0, 0, 0, 0))
}

/// Formats bytes per second into a human-readable string (KB, MB, GB)
/// Note: "/s" is not included - it's shown in the column header instead
pub fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    format_byte_value(bytes_per_sec, "", true, (2, 2, 1, 0))
}

/// Formats the time elapsed since container creation
pub fn format_time_elapsed(created: Option<&chrono::DateTime<Utc>>) -> String {
    match created {
        Some(created_time) => {
            let formatter = Formatter::new();
            let now = Utc::now();
            formatter.convert_chrono(*created_time, now)
        }
        None => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(1), "1 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1 K");
        assert_eq!(format_bytes(1536), "2 K"); // 1.5KB rounds to 2K
        assert_eq!(format_bytes(10240), "10 K");
        assert_eq!(format_bytes(1048575), "1024 K"); // Just under 1MB
    }

    #[test]
    fn test_format_bytes_megabytes() {
        assert_eq!(format_bytes(1048576), "1 M"); // Exactly 1MB
        assert_eq!(format_bytes(536870912), "512 M");
        assert_eq!(format_bytes(1073741823), "1024 M"); // Just under 1GB
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        assert_eq!(format_bytes(1073741824), "1 G"); // Exactly 1GB
        assert_eq!(format_bytes(4294967296), "4 G"); // 4GB
        assert_eq!(format_bytes(17179869184), "16 G"); // 16GB
    }

    #[test]
    fn test_format_bytes_per_sec() {
        assert_eq!(format_bytes_per_sec(0.0), "0 B");
        assert_eq!(format_bytes_per_sec(512.0), "512 B");
        assert_eq!(format_bytes_per_sec(1024.0), "1.0 KB");
        assert_eq!(format_bytes_per_sec(1048576.0), "1.00 MB");
        assert_eq!(format_bytes_per_sec(1073741824.0), "1.00 GB");
    }
}
