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
}
```

And then you can start the optimizer via ....

### Advanced protocol

To be a better behaved optimizer, we encourage you to implement the advanced protocol features.

If you are processing a graph, and you suddenly receive a new graph:
- Immediately finish the old graph and print it out.
- To implement this, run the optimizer and the `stdin.read_line()` loop on a separate threads. `read_line` is a blocking API in most languages.

While processing a graph:
- You can return an optimized graph multiple times. Only the latest output is counted.

## Resources

- https://github.com/jw1912/SPRT