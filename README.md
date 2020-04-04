# Gizmo DB for WebAssembly

## An embedded graph database with a powerful query engine. 

A Rust port of the Cayley Graph Database query engine. Initially this will be a queryable graph database targeting web assembly. But potentially it could be modified to be used as an cross-platform embedable graph database if a Javascript runtime was included, such as Deno. It is currently structured as a hybrid web application to simplify testing and debugging during development. Later it will be restructured to work as an npm package.