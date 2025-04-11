use std::collections::HashMap;

use super::run_statistics::RunStatistics;
use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Feature, Grid, Legend, Toolbox, ToolboxDataZoom},
    datatype::{CompositeValue, DataFrame, DataPoint},
    df,
    element::{AxisType, Label, SplitArea, Tooltip},
    series::{Heatmap, Scatter},
};

pub fn plot_runs(runs: RunStatistics) {
    let graph_names = get_graph_names(&[&runs]);
    let graph_ids = make_graph_ids(&graph_names);
    let best_crossing_values = get_best_crossing_values(&[&runs], &graph_ids);

    let mut chart = Chart::new()
        .legend(Legend::new())
        .tooltip(Tooltip::new())
        .toolbox(
            Toolbox::new()
                .left("center")
                .feature(Feature::new().data_zoom(ToolboxDataZoom::new())),
        )
        .grid(Grid::new().right("57%").bottom("57%"))
        .grid(Grid::new().left("57%").bottom("57%"))
        .grid(Grid::new().right("57%").top("57%"))
        .grid(Grid::new().left("57%").top("57%"))
        .x_axis(
            Axis::new()
                .grid_index(0)
                .scale(true)
                .name("Timestamp of run"),
        )
        .y_axis(
            Axis::new()
                .grid_index(0)
                .scale(true)
                .name("Max Edge Crossings"),
        )
        .x_axis(Axis::new().grid_index(1).scale(true).name("Unused"))
        .y_axis(Axis::new().grid_index(1).scale(true).name("Unused"))
        .x_axis(
            Axis::new()
                .grid_index(2)
                .type_(AxisType::Category)
                .data((0..runs.runs.len()).map(|v| v.to_string()).collect())
                .split_area(SplitArea::new().show(true))
                .name("Run #"),
        )
        .y_axis(
            Axis::new()
                .grid_index(2)
                .type_(AxisType::Category)
                .data(graph_names)
                .split_area(SplitArea::new().show(true))
                .name("Graph Name"),
        )
        .x_axis(Axis::new().grid_index(3).scale(true).name("Unused"))
        .y_axis(Axis::new().grid_index(3).scale(true).name("Unused"));

    {
        let data: Vec<DataPoint> = runs
            .runs
            .iter()
            .flat_map(|run| {
                let timestamp = run.unix_seconds;
                run.graphs
                    .iter()
                    .map(|v| {
                        (v.max_per_edge() as f64)
                            / (best_crossing_values[graph_ids[&v.graph]] as f64)
                    })
                    .map(move |v| {
                        DataPoint::Value(CompositeValue::Array(vec![
                            CompositeValue::from(timestamp as i64), // Good enough
                            CompositeValue::from(v),
                        ]))
                    })
            })
            .collect();
        chart = chart.series(Scatter::new().x_axis_index(0).y_axis_index(0).data(data));
    }

    {
        let heatmap_df: Vec<DataFrame> = runs
            .runs
            .iter()
            .enumerate()
            .flat_map(|(run_id, run)| {
                let graph_ids = &graph_ids;
                run.graphs.iter().map(move |v| {
                    df![
                        CompositeValue::from(run_id as i64),
                        CompositeValue::from(graph_ids[&v.graph] as i64),
                        CompositeValue::from(v.max_per_edge() as i64)
                    ]
                })
            })
            .collect::<Vec<_>>();

        chart = chart.series(
            Heatmap::new()
                .x_axis_index(2)
                .y_axis_index(2)
                .name("Max crossings")
                .label(Label::new().show(true))
                .data(heatmap_df),
        )
    }

    let mut renderer = HtmlRenderer::new("Leaderboard", 1000, 800);
    renderer
        .save(&chart, "./leaderboard/leaderboard.html")
        .expect("chart should be saved to ./leaderboard");

    println!("Latest leadeboard saved to ./leaderboard/leaderboard.html");
}

fn get_graph_names(all_runs: &[&RunStatistics]) -> Vec<String> {
    let mut graph_names = all_runs
        .iter()
        .flat_map(|runs| {
            runs.runs
                .iter()
                .flat_map(|r| r.graphs.iter().map(|g| g.graph.clone()))
        })
        .collect::<Vec<_>>();

    // Sort for displaying
    // I could use
    // https://docs.rs/icu_collator/latest/icu_collator/
    // but that seemed heavyweight, so I went with an alternative
    numeric_sort::sort_unstable(&mut graph_names);

    graph_names
}

fn make_graph_ids(graph_names: &[String]) -> HashMap<String, usize> {
    graph_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect()
}

fn get_best_crossing_values(
    all_runs: &[&RunStatistics],
    graph_ids: &HashMap<String, usize>,
) -> Vec<u32> {
    let mut best_values = vec![u32::MAX; graph_ids.len()];
    for graph in all_runs
        .iter()
        .flat_map(|r| &r.runs)
        .flat_map(|r| &r.graphs)
    {
        let id = graph_ids[&graph.graph] as usize;
        best_values[id] = best_values[id].min(graph.max_per_edge());
    }
    best_values
}
