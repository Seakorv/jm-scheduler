//! Benchmarks for `evaluate` and `candidates` on an event-sized project (task 4.21).
//!
//! Run with `cargo bench -p jmetro-core`.

use std::hint::black_box;
use std::num::NonZeroU32;

use chrono::NaiveDate;
use criterion::{Criterion, criterion_group, criterion_main};
use jmetro_core::{
    AreaId, Line, Location, LocationKind, Minutes, Project, TimeOfDay, TravelTime, Visit,
    candidates, evaluate,
};

const BARS: u32 = 20;
const LINES: u32 = 30;
const STOPS: u32 = 7;

/// 20 bars with travel times between every pair, and 30 lines × 7 stops.
/// Each line visits every third bar from its own starting bar, so no line repeats a bar,
/// and departures are spread over an hour like at a real event.
fn generated_project() -> Project {
    let date = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
    let mut project = Project::new(String::from("Benchmark"), date);
    project.settings.stops_per_line = STOPS;

    let bar_ids: Vec<_> = (0..BARS)
        .map(|i| {
            let bar = Location::new(
                format!("Bar {i}"),
                LocationKind::Bar {
                    capacity: NonZeroU32::new(3).unwrap(),
                    area_id: AreaId::new(),
                    overload: None,
                },
                String::new(),
                None,
                None,
            );
            let id = bar.id;
            project.locations.push(bar);
            id
        })
        .collect();

    let mut stops = vec![project.start_location_id];
    stops.extend(&bar_ids);
    stops.push(project.end_location_id);
    for (i, &from_id) in stops.iter().enumerate() {
        for (j, &to_id) in stops.iter().enumerate().skip(i + 1) {
            project.travel_times.push(TravelTime {
                from_id,
                to_id,
                minutes: Minutes::new(5 + u32::try_from(i * j % 15).unwrap()),
                distance_km: None,
            });
        }
    }

    for n in 0..LINES {
        let departure = TimeOfDay::new(16 * 60 + n % 6 * 10).unwrap();
        let mut line = Line::new(n + 1, None, false, departure, None);
        for k in 0..STOPS {
            let bar = (n + k * 3) % BARS;
            line.visits.push(Visit {
                location_id: bar_ids[bar as usize],
                stay_time: Minutes::new(40),
            });
        }
        project.lines.push(line);
    }

    project
}

fn bench(c: &mut Criterion) {
    let project = generated_project();
    let line_id = project.lines[0].id;

    c.bench_function("evaluate 30x7", |b| {
        b.iter(|| evaluate(black_box(&project)));
    });
    c.bench_function("candidates 30x7, 20 bars", |b| {
        b.iter(|| candidates(black_box(&project), line_id, 0));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
