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

## How to make a good tester

We start up the optimizer once. Extra commandline arguments are sent to the optimizer. This is to make it easy to test different variants of an optimizer.
Some programming languages have very long and slow startup times.

The protocol should be relatively simple. A stdin-stdout protocol is reasonably popular for this. Even language servers use it.
The biggest challenge with that protocol is remembering to print debug info to stderr!

Then, the optimizer should send
- its name
- its version (e.g. a version number, or a git commit sha)
- its settings (set via extra commandline arguments)

This is used for keeping track of which optimizer combinations have been tried out already.

At this point, we query
- the filesystem for all graphs that we could send. We sort them alpha-numerically
- the past runs. If we already have a result for a (graph name, optimizer name, version, settings), then we skip that graph.

And the following steps happen in a loop:

The optimizer should send an "input graph" request.
The tester responds with a graph, or closes the pipe if all graphs are done.

The optimizer then
- uses a hardcoded seed
- and obeys a hardcoded timeout
and responds with an optimized graph

The tester 
- checks if the solution graph is valid
- computes the max edge crossings
- and stores the info in a CSV file, or another file format that can easily be imported into Excel. The file format should be plaintext file so that git can work with it.




