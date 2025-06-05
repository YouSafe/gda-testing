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
// Print the name of
// - your solver
// - a version number
// - and what parameters its being executed with
println!("START Team1-v4-spring-only");
while(true) {
    // Request an instance
    println!("GRAPH");

    // An entire graph on one line
    let json_graph = stdin.read_line(); 
    
    // Check for stdin being closed. This is programming language specific.
    if json_graph.len() == 0 { 
        break;
    }

    println!(json_graph_optimized);
}
```

Your optimizer first announces its name. That name will be used for the output `.csv` file.

Then, it requests a JSON graph. It'll be formatted on a single line as an input.
Your optimizer can now run its algorithm(s) on the graph. Please use a timeout here, you don't want the testing tool to hang forever.

Finally, it should print the entire resulting graph, formatted on a single line.
Remember to have a line break at the end.

The testing tool will *validate* that your output graph is valid, and will also compute the crossings.
This can be extremely helpful for spotting bugs.

Optimizers can print debug information to `stderr`. This will show up in the console.
I recommend printing a lot of useful info there.

For your convenience, any print statements other than `START` and `GRAPH` won't have an effect.

## Commit your results!

We encourage you to send us your results! Send us a GitHub pull request, and we'll add them.

## Resources

- https://github.com/jw1912/SPRT

## For the future: How to make a good tester

We start up the optimizer once. Extra commandline arguments are sent to the optimizer. This is to make it easy to test different variants of an optimizer.
Some programming languages have very long and slow startup times.

The protocol should be relatively simple. A stdin-stdout protocol is reasonably popular for this. Even language servers use it.
The biggest challenge with that protocol is remembering to print debug info to stderr!

Then, the optimizer should send
- its name
- its version (e.g. a version number, or a git commit sha)
- its settings (set via extra commandline arguments)

This is used for keeping track of which optimizer combinations have been tried out already.

<!--
At this point, we query
- the filesystem for all graphs that we could send. We sort them alpha-numerically
- the past runs. If we already have a result for a (graph name, optimizer name, version, settings), then we skip that graph.
-->

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




