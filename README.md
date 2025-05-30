# GDA - Testing

Testing framework for testing different versions of optimizers.

![Screenshot of the leaderboard](./leaderboard.png)

## Test Instances

All the test instances are in the [./graphs](./graphs) folder.

A description of them can be found in [GRAPHS.md](./GRAPHS.md)

## Test Runner

Then run `cargo run graphs 'path/to/your/optimizer'`

If your optimizer requires a complex command, make sure to use quotes `cargo run graphs 'complex command --with --args`.

This generates a `stats/optimizer-name.csv` file with some statistics.

`cargo run leaderboard` takes those files and generates a leaderboard out of them!


## Protocol for optimizers

So you're writing an optimizer and want to use the automated testing infrastructure?

Just add a main loop like this to your program
```rs
println!("START Solver-Name-And-Version-Goes-Here");
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
    }
    println!("DONE");
}
```

Your optimizer first announces its name. e.g. `START team-1-v0.0.1`

Then, it repeatedly gets a JSON graph formatted on a single line as an input.
Your optimizer can now run its algorithm(s) on the graph. And it outputs a graph for each of those.
e.g. If you are running it with two spring based layouts, and then with a force based layout, you could output
```
GRAPH spring-25
{ "edges": [...], "nodes": [...], "width": ..., "height": ... }
GRAPH spring-75
{ ... }
GRAPH force-25
```

Finally, the optimizer announces that it's ready for the next graph by printing `DONE`.

Optimizers can print debug information to `stderr`. This will show up in the console.
I recommend printing a lot of useful info there.

## Resources

- https://github.com/jw1912/SPRT



