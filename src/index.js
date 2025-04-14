const data = await fetch("./leaderboard.json")
  .then((v) => v.json())
  .catch((err) => {
    console.error(err);
    document.body.prepend(
      document.createTextNode(
        "Missing leaderboard JSON. Try running `cargo run plots`"
      )
    );
  });
console.log({ data });
function createNode(tagName, opts, children = []) {
  const node = document.createElement(tagName);
  if (opts.id) {
    node.id = opts.id;
  }
  if (opts.class) {
    node.className = opts.class;
  }
  if (opts.style) {
    Object.entries(opts.style).forEach(([name, value]) => {
      node.style[name] = value;
    });
  }
  if (opts.attributes) {
    opts.attributes.forEach(([name, value]) => {
      node.setAttribute(name, value);
    });
  }
  if (opts.listeners) {
    opts.listeners.forEach(([name, value]) => {
      node.addEventListener(name, value);
    });
  }
  node.append(...children);
  return node;
}
for (const runs of data.all_runs) {
  document.body.append(
    createNode("div", {}, [
      createNode("h1", {}, [document.createTextNode(runs.name)]),
      createNode("h2", {}, [document.createTextNode("Results")]),
    ])
  );
}
const chart = echarts.init(document.getElementById("chart"), null, {
  renderer: "canvas",
});
const option = {
  tooltip: {},
  legend: {},
  toolbox: {
    feature: {
      dataZoom: {},
    },
    left: "center",
  },
  grid: [
    {
      right: "57%",
      bottom: "57%",
    },
    {
      left: "57%",
      bottom: "57%",
    },
    {
      top: "57%",
      right: "57%",
    },
    {
      left: "57%",
      top: "57%",
    },
  ],
  xAxis: [
    {
      gridIndex: 0.0,
      name: "Timestamp of run",
      scale: true,
    },
    {
      gridIndex: 1.0,
      name: "Unused",
      scale: true,
    },
    {
      type: "category",
      gridIndex: 2.0,
      name: "Run #",
      splitArea: {
        show: true,
      },
      data: ["0"],
    },
    {
      gridIndex: 3.0,
      name: "Unused",
      scale: true,
    },
  ],
  yAxis: [
    {
      gridIndex: 0.0,
      name: "Max Edge Crossings",
      scale: true,
    },
    {
      gridIndex: 1.0,
      name: "Unused",
      scale: true,
    },
    {
      type: "category",
      gridIndex: 2.0,
      name: "Graph Name",
      axisLabel: {
        fontSize: 8.0,
      },
      splitArea: {
        show: true,
      },
      data: [
        "/example-instances-2024/150-nodes.json",
        "/example-instances-2024/sol-150-nodes-50-planar.json",
      ],
    },
    {
      gridIndex: 3.0,
      name: "Unused",
      scale: true,
    },
  ],
  series: [
    {
      type: "scatter",
      xAxisIndex: 0.0,
      yAxisIndex: 0.0,
      data: [
        [1744622856, 1.0],
        [1744622856, 1.0],
      ],
    },
    {
      type: "heatmap",
      name: "Max crossings",
      xAxisIndex: 2.0,
      yAxisIndex: 2.0,
      label: {
        show: true,
      },
      data: [
        [0, 0, 49],
        [0, 1, 49],
      ],
    },
  ],
};
chart.setOption(option);
