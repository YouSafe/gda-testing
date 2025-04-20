# GDA - Testing

Testing framework for testing different versions of optimizers.

TODO: Put screenshot of the beautiful optimizer statistics here

## Protocol for optimizers

So you're writing an optimizer and want to use this?

Just add a main loop like this to your program
```rs
while(true) {
    // Read a single line from the input
    let json_graph = match stdin.read_line() {
        Ok(line) => line, // Each line is an entire graph
        Err(_) => break // And when the stream is closed, we exit. Some languages return an empty string when stdin is closed.
    };

    // TODO: Parse the graph
    // TODO: Optimize the graph
    // TODO: Serialize the graph

    // Print the optimized graph back, and be ready for the next input
    println!(json_graph_optimized);
    println!("DONE");
}
```

And then you can start the optimizer via `cargo run leaderboard 'name' 'path/to/optimizer'`.
If your optimizer requires a complex command, make sure to use quotes `cargo run leaderboard 'name' 'complex command --with --args`.

### Advanced Protocol

Input 
- `{ a JSON graph }\n` starts your optimizer.
Output
- `{ an optimized JSON graph }\n` records a result, can be used multiple times.
- `DONE\n` says that you are done with optimizing. Now you can receive a new input graph.

Good optimizers will try to print out a JSON graph every ~30 seconds.
This will be recorded and used for the visualisation. That makes comparing long running optimizers easier.

Optimizers can print debug information to `stderr`. This will show up in the console.

## Resources

- https://github.com/jw1912/SPRT

## Test Graphs

- `graphdrawingcontest` has all graphs from https://graphdrawingcontest.appspot.com/input.jsp
  - `automaticcheck-3` had a duplicated ID, this was fixed
  - The following have been excluded
    - `manual` has graphs that are covered by automatic
    - `automaticcheck` has graphs that are covered by automatic
    - `test-5` was a duplicate of `test-4`
- `example-instances-2024` was provided by our tutor
- Bipartite
- K-Partite: Bipartite but with more than 2 groups
- Complete
- [Cube](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gab7c85da1b67c5f397be073826a532f39)
- [Globe](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga286a9b4e6d5f2feedb286585176ca628)
- [No edges](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga44c2631acd39f73c7117a8a8c60d6071)
- Grid
- [Toroidal Grid](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga5e5147b533c68c25f3372b3ec5c2f04b)
- [Lattice](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga6c2abcc203dcfc0839f5233afeebbe5d)
- Waxman graph: Random euclidean
- Petersen
- Tree 
- Line: (Tree with child count = 1)
- Wheel
- Circulant Graph
- [Random](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga0feff1b510864aba8b73a1b34e5f2ca1)
- [Random simple connected](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga781aeb9ae0e597beb8cfd97f2dc15201)
- [Random simple](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gac6991a8ef695dc1ce1c320aeb843856d)
- [Random hierarchical](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gaa4f8e06a35368a8ce24efcbb71bf1e36)
- [Random planar](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gae9de58fd22ae2533f0d81d450d4bf985)
- [Random planar tri-connected](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga38b77440e49db5110960a11be8195a30)
- [Regular](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga5e2b0644b941d5f8bb7770a27a1f6171)
- [Random series parallel](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gab7bf735577889df8926599b1152957f5)
- [Erdős-Rényi](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga066156d279149423d377f108d42b19c1)
- [Random tree](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gad83c1576fee773abf95334f842f6849b)
- [Watts & Strogatz](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#gaba4c92aaba97ed6ec8c4e250268f5c5a)

- [Dorogovtsev-Mendes](https://juliagraphs.org/Graphs.jl/stable/core_functions/simplegraphs_generators/#Graphs.SimpleGraphs.dorogovtsev_mendes-Tuple{Integer})


We also need
connected clusters
star graph 

### Test Graph Modifications

- [Preferential Attachment](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga70be73bda36b4aeb89122bcd3154af7c)
- [Suspension](https://ogdf.github.io/doc/ogdf/group__graph-generators.html#ga3b6fc792acfc6697de0ae62c01df372b)
- Series parallel composition
- Multiple disconnected graphs
- Combine multiple graphs with edges
- Add/remove random edges


### OGDF also has these
Am unsure about those
https://ogdf.github.io/doc/ogdf/group__graph-generators.html
- randomBiconnectedGraph
- randomChungLuGraph
- randomClusterGraph
- randomEdgesGraph
- randomGeographicalThresholdGraph
- randomGeometricCubeGraph


