# GDA - Testing

Testing framework for testing different versions of optimizers.

TODO: Put screenshot of the beautiful optimizer statistics here

## Protocol for optimizers

So you're writing an optimizer and want to use this?

Just add a main loop like this to your program
```rs
println!("START Solver-Name-Goes-Here");
while(true) {
    // Read a single line from the input
    let json_graph = match stdin.read_line() {
        Ok(line) => line, // Each line is an entire graph
        Err(_) => break // And when the stream is closed, we exit. Some languages return an empty string when stdin is closed.
    };

    for parameter in parameters {
      // TODO: Parse the graph
      // TODO: Optimize the graph
      // TODO: Serialize the graph

      // Print the optimized graph back, and be ready for the next input
      println!("GRAPH Solver-Parameters-Go-Here");
      println!(json_graph_optimized);
      println!("DONE");
    }
}
```

And then you can start the optimizer via `cargo run graphs 'path/to/optimizer'`.
If your optimizer requires a complex command, make sure to use quotes `cargo run graphs 'complex command --with --args`.

### Protocol Description

Your solver first announces its name. e.g. `START team-1`

Then, it repeatedly gets a JSON graph formatted on a single line as an input.
Your solver can now run its algorithm(s) on the graph. And it outputs a graph for each of those.
e.g. If you are running it with two spring based layouts, and then with a force based layout, you could output
`GRAPH spring-25\n`, `{ ... }`, `GRAPH spring-75`, `{ ... }`, `GRAPH force-25`, `{ .... }`

Finally, the solver announces that it's ready for the next graph by printing `DONE`.

Optimizers can print debug information to `stderr`. This will show up in the console.
I recommend printing a lot of useful info there.

## Resources

- https://github.com/jw1912/SPRT

## Test Graphs

- https://graphdrawingcontest.appspot.com/input.jsp
  - `manual` has graphs that are covered by automatic
  - `automaticcheck` has graphs that are covered by automatic
  - `automaticcheck-3` had a duplicated ID, this was fixed
  - `test-5` was a duplicate of `test-4`
- `example-instances-2024` was provided by our tutor
