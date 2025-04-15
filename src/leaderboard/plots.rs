use super::stats::TeamStats;
use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Feature, Legend, Toolbox, ToolboxDataZoom},
    datatype::{CompositeValue, DataPoint},
    element::{AxisType, Formatter, Tooltip},
    series::Scatter,
};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

pub fn plot_leaderboard(all_teams: Vec<TeamStats>) -> std::io::Result<()> {
    let graph_names = get_graph_names(&all_teams);
    let graph_ids = make_graph_ids(&graph_names);
    let best_crossing_values: Vec<u32> = get_best_crossing_values(&all_teams, &graph_ids)
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    let team_names = all_teams.iter().map(|v| v.name.clone()).collect::<Vec<_>>();
    let mut chart = Chart::new()
        .legend(Legend::new())
        .tooltip(
            Tooltip::new().formatter(Formatter::Function(
                "(params) => {
            let score = params.data[1].toLocaleString(undefined, { minimumFractionDigits: 2 });
            let crossings = params.data[2];
            let graph = params.data[3];
            return `${crossings} crossings on ${graph}<br>(score ${score})`;
          }"
                .into(),
            )),
        )
        .toolbox(
            Toolbox::new()
                .left("center")
                .feature(Feature::new().data_zoom(ToolboxDataZoom::new())),
        )
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(team_names)
                .name("Team"),
        )
        .x_axis(Axis::new().min(0).max(all_teams.len() as i32).show(false))
        .y_axis(Axis::new().min(0.0).max(1.0).name("Relative score"));

    let mut data: Vec<DataPoint> = vec![];
    for (team_id, team) in all_teams.iter().enumerate() {
        let best_crossing_values = &best_crossing_values;
        let graph_names = &graph_names;
        let crossing_values = get_best_crossing_values(std::iter::once(team), &graph_ids);
        println!("{:?}", crossing_values);
        let scores = crossing_values
            .iter()
            .enumerate()
            .filter_map(|(graph_id, v)| v.map(move |v| (graph_id, v)))
            .map(move |(graph_id, v)| {
                let relative_score = (best_crossing_values[graph_id] as f32) / (v as f32);
                DataPoint::Value(CompositeValue::Array(vec![
                    CompositeValue::from(team_id as f64 + 0.5 + random_scatter(graph_id) * 0.1),
                    CompositeValue::from(relative_score),
                    // And here we store some extra data for hover texts
                    CompositeValue::from(v as i64),
                    CompositeValue::from(graph_names[graph_id].clone()),
                ]))
            });
        data.extend(scores);
    }
    chart = chart.series(Scatter::new().x_axis_index(1).y_axis_index(0).data(data));
    let mut renderer = HtmlRenderer::new("Leaderboard", 1000, 760);
    renderer
        .save(&chart, "./leaderboard.html")
        .expect("chart should be saved to ./leaderboard");
    println!("Generated leaderboard. Open ./leaderboard.html in a browser!");
    Ok(())
}

fn get_graph_names(all_runs: &[TeamStats]) -> Vec<String> {
    let mut graph_names = all_runs
        .iter()
        .flat_map(|runs| runs.runs.iter().map(|r| r.graph.clone()))
        .collect::<Vec<_>>();

    // Sort for displaying
    // I could use
    // https://docs.rs/icu_collator/latest/icu_collator/
    // but that seemed heavyweight, so I went with an alternative
    numeric_sort::sort_unstable(&mut graph_names);

    // And now that it's sorted we can efficiently remove duplicates
    graph_names.dedup();

    graph_names
}

fn random_scatter(input: usize) -> f64 {
    rand::rngs::SmallRng::seed_from_u64(input as u64).r#gen()
}

fn make_graph_ids(graph_names: &[String]) -> HashMap<String, usize> {
    graph_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect()
}

fn get_best_crossing_values<'a>(
    all_runs: impl IntoIterator<Item = &'a TeamStats>,
    graph_ids: &HashMap<String, usize>,
) -> Vec<Option<u32>> {
    let mut best_values: Vec<Option<u32>> = vec![None; graph_ids.len()];
    for graph in all_runs.into_iter().flat_map(|r| &r.runs) {
        let id = graph_ids[&graph.graph] as usize;
        if let Some(v) = &mut best_values[id] {
            *v = (*v).min(graph.max_per_edge);
        } else {
            best_values[id] = Some(graph.max_per_edge);
        }
    }
    best_values
}
