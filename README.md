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

- https://graphdrawingcontest.appspot.com/input.jsp
  - `manual` has graphs that are covered by automatic
  - `automaticcheck` has graphs that are covered by automatic
  - `automaticcheck-3` had a duplicated ID, this was fixed
  - `test-5` was a duplicate of `test-4`
- `example-instances-2024` was provided by our tutor
