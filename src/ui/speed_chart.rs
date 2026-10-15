use std::collections::VecDeque;
use std::time::{Duration, Instant};

use iced::advanced::text as advanced_text;
use iced::widget::canvas;
use iced::widget::canvas::{Frame, Geometry, Path, Program, Stroke};
use iced::widget::{Space, column, container, row, text};
use iced::{Color, Element, Length, Pixels, Point, Rectangle, Renderer, Theme, mouse};

use crate::app::Message;
use crate::ui::theme;

/// Number of `>BYTECOUNT` samples kept in the rolling window. The management
/// socket is configured with `bytecount 5`, so 60 samples cover 5 minutes.
const MAX_SAMPLES: usize = 60;
const CHART_HEIGHT: f32 = 100.0;
/// Lowest y-axis scale so an idle connection still shows a meaningful baseline.
const MIN_SCALE: f32 = 1024.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct SpeedSample {
    pub download: f32,
    pub upload: f32,
}

/// Rolling window of per-second throughput derived from cumulative byte counters.
#[derive(Debug, Default)]
pub struct SpeedHistory {
    samples: VecDeque<SpeedSample>,
    last: Option<(u64, u64, Instant)>,
}

impl SpeedHistory {
    pub fn reset(&mut self) {
        self.samples.clear();
        self.last = None;
    }

    /// Feed the latest cumulative counters; the delta since the previous call
    /// becomes the next sample.
    pub fn record(&mut self, bytes_in: u64, bytes_out: u64) {
        self.record_at(bytes_in, bytes_out, Instant::now());
    }

    fn record_at(&mut self, bytes_in: u64, bytes_out: u64, now: Instant) {
        if let Some((prev_in, prev_out, prev_at)) = self.last
            && let Some(sample) = rate_sample(bytes_in, bytes_out, prev_in, prev_out, now - prev_at)
        {
            self.samples.push_back(sample);
            while self.samples.len() > MAX_SAMPLES {
                self.samples.pop_front();
            }
        }
        self.last = Some((bytes_in, bytes_out, now));
    }

    pub fn latest(&self) -> SpeedSample {
        self.samples.back().copied().unwrap_or_default()
    }
}

fn rate_sample(
    in_now: u64,
    out_now: u64,
    in_prev: u64,
    out_prev: u64,
    elapsed: Duration,
) -> Option<SpeedSample> {
    let secs = elapsed.as_secs_f32();
    if secs <= 0.0 {
        return None;
    }
    Some(SpeedSample {
        // Counters drop back to zero when OpenVPN re-negotiates; a negative
        // delta is a reset, not a burst of negative traffic.
        download: if in_now >= in_prev {
            (in_now - in_prev) as f32 / secs
        } else {
            0.0
        },
        upload: if out_now >= out_prev {
            (out_now - out_prev) as f32 / secs
        } else {
            0.0
        },
    })
}

pub fn view<'a>(history: &'a SpeedHistory) -> Element<'a, Message> {
    let latest = history.latest();

    let legend = |arrow: &str, value: f32, color: fn(&Theme) -> Color| {
        text(format!("{arrow} {}", format_speed(value)))
            .size(11)
            .font(theme::MONO)
            .style(move |t: &Theme| iced::widget::text::Style {
                color: Some(color(t)),
            })
    };

    let header = row![
        text("NETWORK SPEED \u{00B7} 5 MIN")
            .size(10)
            .style(theme::text_muted),
        Space::new().width(Length::Fill),
        legend("\u{2193}", latest.download, theme::info_accent),
        legend("\u{2191}", latest.upload, theme::success),
    ]
    .spacing(f32::from(theme::SPACE_MD))
    .align_y(iced::Alignment::Center);

    let chart = canvas(SpeedChart {
        samples: &history.samples,
    })
    .width(Length::Fill)
    .height(Length::Fixed(CHART_HEIGHT));

    container(
        column![header, chart]
            .spacing(f32::from(theme::SPACE_SM))
            .width(Length::Fill),
    )
    .padding([theme::SPACE_SM + 2, theme::SPACE_MD + 2])
    .width(Length::Fill)
    .style(theme::card_filled)
    .into()
}

struct SpeedChart<'a> {
    samples: &'a VecDeque<SpeedSample>,
}

impl<'a> Program<Message> for SpeedChart<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let plot = Rectangle {
            x: 4.0,
            y: 10.0,
            width: (bounds.width - 8.0).max(1.0),
            height: (bounds.height - 14.0).max(1.0),
        };

        let muted = theme::muted(theme);

        let grid = Color { a: 0.18, ..muted };
        for step in 0..=3 {
            let y = plot.y + plot.height * (step as f32 / 3.0);
            frame.stroke(
                &Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)),
                Stroke::default().with_width(1.0).with_color(grid),
            );
        }

        if self.samples.is_empty() {
            frame.fill_text(canvas::Text {
                content: "Waiting for data\u{2026}".to_string(),
                position: Point::new(plot.x + plot.width / 2.0, plot.y + plot.height / 2.0),
                color: Color { a: 0.8, ..muted },
                size: Pixels(11.0),
                align_x: advanced_text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
            return vec![frame.into_geometry()];
        }

        let y_max = nice_ceil(
            self.samples
                .iter()
                .map(|s| s.download.max(s.upload))
                .fold(0.0, f32::max),
        );

        let download_color = theme::info_accent(theme);
        let upload_color = theme::success(theme);

        if let Some(area) = area_path(self.samples, plot, y_max, |s| s.download) {
            frame.fill(
                &area,
                Color {
                    a: 0.12,
                    ..download_color
                },
            );
        }
        if let Some(area) = area_path(self.samples, plot, y_max, |s| s.upload) {
            frame.fill(
                &area,
                Color {
                    a: 0.10,
                    ..upload_color
                },
            );
        }

        frame.stroke(
            &series_path(self.samples, plot, y_max, |s| s.download),
            line_stroke(download_color),
        );
        frame.stroke(
            &series_path(self.samples, plot, y_max, |s| s.upload),
            line_stroke(upload_color),
        );

        frame.fill_text(canvas::Text {
            content: format_speed(y_max),
            position: Point::new(plot.x + 2.0, plot.y + 1.0),
            color: Color { a: 0.75, ..muted },
            size: Pixels(9.0),
            font: theme::MONO,
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }
}

fn line_stroke(color: Color) -> Stroke<'static> {
    Stroke::default()
        .with_width(1.6)
        .with_color(color)
        .with_line_cap(canvas::LineCap::Round)
        .with_line_join(canvas::LineJoin::Round)
}

/// The newest sample sits at the right edge; a single sample is drawn as a
/// full-width level line.
fn series_path(
    samples: &VecDeque<SpeedSample>,
    plot: Rectangle,
    y_max: f32,
    pick: fn(&SpeedSample) -> f32,
) -> Path {
    Path::new(|builder| {
        let len = samples.len();
        if len == 1 {
            let y = plot.y + plot.height * (1.0 - (pick(&samples[0]) / y_max).clamp(0.0, 1.0));
            builder.move_to(Point::new(plot.x, y));
            builder.line_to(Point::new(plot.x + plot.width, y));
            return;
        }
        for (index, sample) in samples.iter().enumerate() {
            let point = Point::new(
                plot.x + plot.width * (index as f32 / (len - 1) as f32),
                plot.y + plot.height * (1.0 - (pick(sample) / y_max).clamp(0.0, 1.0)),
            );
            if index == 0 {
                builder.move_to(point);
            } else {
                builder.line_to(point);
            }
        }
    })
}

fn area_path(
    samples: &VecDeque<SpeedSample>,
    plot: Rectangle,
    y_max: f32,
    pick: fn(&SpeedSample) -> f32,
) -> Option<Path> {
    if samples.len() < 2 {
        return None;
    }
    Some(Path::new(|builder| {
        let len = samples.len();
        for (index, sample) in samples.iter().enumerate() {
            let point = Point::new(
                plot.x + plot.width * (index as f32 / (len - 1) as f32),
                plot.y + plot.height * (1.0 - (pick(sample) / y_max).clamp(0.0, 1.0)),
            );
            if index == 0 {
                builder.move_to(point);
            } else {
                builder.line_to(point);
            }
        }
        builder.line_to(Point::new(plot.x + plot.width, plot.y + plot.height));
        builder.line_to(Point::new(plot.x, plot.y + plot.height));
        builder.close();
    }))
}

/// Round a scale up to a value that formats cleanly with binary (1024-based)
/// units: 1, 2, 5, 10, 20, … × 1024^n.
fn nice_ceil(value: f32) -> f32 {
    const STEPS: [f32; 10] = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0];

    let value = value.max(MIN_SCALE);
    let magnitude = 1024f32.powf(value.log(1024.0).floor());
    let normalized = value / magnitude;
    let step = STEPS
        .iter()
        .copied()
        .find(|candidate| normalized <= *candidate)
        .unwrap_or(1024.0);
    step * magnitude
}

fn format_speed(bytes_per_sec: f32) -> String {
    const KB: f32 = 1024.0;
    const MB: f32 = 1024.0 * KB;

    if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0.0), "0 B/s");
        assert_eq!(format_speed(1536.0), "1.5 KB/s");
        assert_eq!(format_speed(1024.0 * 1024.0 * 2.5), "2.5 MB/s");
    }

    #[test]
    fn test_nice_ceil() {
        assert_eq!(nice_ceil(0.0), 1024.0);
        assert_eq!(nice_ceil(1024.0), 1024.0);
        assert_eq!(nice_ceil(1500.0), 2048.0);
        assert_eq!(nice_ceil(130_000.0), 204_800.0);
        assert_eq!(nice_ceil(9_000_000.0), 10_485_760.0);
    }

    #[test]
    fn test_record_computes_rates() {
        let mut history = SpeedHistory::default();
        let t0 = Instant::now();
        history.record_at(0, 0, t0);
        history.record_at(2048, 1024, t0 + Duration::from_secs(2));

        let latest = history.latest();
        assert!((latest.download - 1024.0).abs() < f32::EPSILON);
        assert!((latest.upload - 512.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_first_record_is_baseline_only() {
        let mut history = SpeedHistory::default();
        history.record_at(100, 200, Instant::now());
        assert!(history.samples.is_empty());
    }

    #[test]
    fn test_record_handles_counter_reset() {
        let mut history = SpeedHistory::default();
        let t0 = Instant::now();
        history.record_at(10_000, 5_000, t0);
        history.record_at(500, 200, t0 + Duration::from_secs(5));

        let latest = history.latest();
        assert_eq!(latest.download, 0.0);
        assert_eq!(latest.upload, 0.0);
    }

    #[test]
    fn test_reset_clears_window() {
        let mut history = SpeedHistory::default();
        let t0 = Instant::now();
        history.record_at(0, 0, t0);
        history.record_at(1024, 1024, t0 + Duration::from_secs(5));
        history.reset();

        assert!(history.samples.is_empty());
        assert!(history.last.is_none());
    }

    #[test]
    fn test_history_caps_at_max_samples() {
        let mut history = SpeedHistory::default();
        let t0 = Instant::now();
        for i in 0..(MAX_SAMPLES + 10) {
            history.record_at(i as u64 * 100, 0, t0 + Duration::from_secs(i as u64));
        }
        assert_eq!(history.samples.len(), MAX_SAMPLES);
    }
}
