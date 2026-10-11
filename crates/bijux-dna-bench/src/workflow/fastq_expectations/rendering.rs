//! Owner: bijux-dna-bench
//! Deterministic rendering of exact FASTQ read-length evidence.

use bijux_dna_science::evidence::{FastqInputLayout, FastqReadLengthTruth};

const SUMMARY_HEADER: &str = "sample_id\tinput_layout\tfile_count\tread_count\tminimum_read_length\tmean_read_length\tmedian_read_length\tmaximum_read_length\tdistinct_read_lengths";
const HISTOGRAM_HEADER: &str = "sample_id\tread_length\tcount";

pub(super) fn render_summary(truth: &FastqReadLengthTruth) -> String {
    format!(
        "{SUMMARY_HEADER}\n{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        truth.sample_id,
        layout_name(truth.input_layout),
        truth.file_count,
        truth.read_count,
        truth.minimum_read_length,
        format_awk_general(truth.mean_read_length),
        format_awk_general(truth.median_read_length),
        truth.maximum_read_length,
        truth.distinct_read_lengths,
    )
}

pub(super) fn render_histogram(truth: &FastqReadLengthTruth) -> String {
    let mut output = format!("{HISTOGRAM_HEADER}\n");
    for (length, count) in &truth.histogram {
        output.push_str(&truth.sample_id);
        output.push('\t');
        output.push_str(&length.to_string());
        output.push('\t');
        output.push_str(&count.to_string());
        output.push('\n');
    }
    output
}

fn layout_name(layout: FastqInputLayout) -> &'static str {
    match layout {
        FastqInputLayout::SingleEnd => "single_end",
        FastqInputLayout::PairedEnd => "paired_end",
    }
}

// FASTQ lengths are bounded below 10^12, where AWK's `%.12g` uses fixed notation.
fn format_awk_general(value: f64) -> String {
    let mut integer_digits = 1_usize;
    let mut threshold = 10.0_f64;
    while value >= threshold && integer_digits < 12 {
        integer_digits += 1;
        threshold *= 10.0;
    }
    let decimal_places = 12_usize.saturating_sub(integer_digits);
    let mut rendered = format!("{value:.decimal_places$}");
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::format_awk_general;

    #[test]
    fn awk_general_format_keeps_twelve_significant_digits_without_padding() {
        assert_eq!(format_awk_general(151.0), "151");
        assert_eq!(format_awk_general(3.5), "3.5");
        assert_eq!(format_awk_general(123.456_789_012_4), "123.456789012");
        assert_eq!(format_awk_general(16_777_216.0), "16777216");
    }
}
