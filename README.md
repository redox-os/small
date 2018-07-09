# small

This is a library that provides "small" types (meaning things are allocated on the stack up to a certain size).

Currently there is only an implementation of a small string.

## Helping out

We'd love to get more help with ensuring that these functions are the most memory efficient and the fastest that they can be. If you can help, that would be fantastic!

There are, however, a few rules when it comes to submitting merge requests:
 1. Ensure that all functions that are publicly facing have sufficient documentation and include at least one doc test.
 2. Ensure that all possible branches through your code is tested, preferably across multiple tests.
 3. Make benchmarks between the old version and the new, and make those readily available.
