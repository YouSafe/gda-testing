use super::run_statistics::RunStatistics;
use charming::{
    Chart, HtmlRenderer,
    component::Axis,
    datatype::{CompositeValue, DataPoint},
    series::Scatter,
};

pub fn plot_runs(runs: RunStatistics) {
    let mut chart = Chart::new()
        .x_axis(Axis::new().scale(true).name("Timestamp of run"))
        .y_axis(Axis::new().scale(true).name("Max Edge Crossings"));

    if let Some(latest_run) = runs.runs.last() {
        let timestamp = latest_run.unix_seconds;
        let data = latest_run
            .graphs
            .iter()
            .map(|v| {
                v.crossings // Hm, those values aren't very compare-able
                    .last()
                    .map(|c| c.max_per_edge as i32)
                    .unwrap_or(-1) // Hm bad
            })
            .map(|v| {
                DataPoint::Value(CompositeValue::Array(vec![
                    CompositeValue::from(v),
                    CompositeValue::from(timestamp as i64), // Good enough
                ]))
            })
            .collect();
        chart = chart.series(Scatter::new().data(data));
    }

    let mut renderer = HtmlRenderer::new("Leaderboard", 1000, 800);
    renderer
        .save(&chart, "./leaderboard/leaderboard.html")
        .expect("chart should be saved to ./leaderboard");

    println!("Latest leadeboard saved to ./leaderboard/leaderboard.html");
}
