
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


