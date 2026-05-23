Multiplecodequiz
=============
A simple online code quiz, mostly AI generated slop.

The idea is simple:
- Take a Rust codebase
- Chunk it up into pieces of code
- For each unique piece, generate a description and embed it in a vector database
- Serve a quiz frontend that presents the user with a piece of code and four possible descriptions from the database, including one correct one from the database

The scoring is completely client-side, no accounts or anything, just a static web page.
