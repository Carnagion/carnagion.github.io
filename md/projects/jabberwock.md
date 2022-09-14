# Jabberwock

*September 2022 - Present* **·** [GitHub](https://github.com/Carnagion/jabberwock) **·** [Crate](https://crates.io/crates/jabberwock)

**Jabberwock** is a static site generator (SSG) I built around the time I wanted to make a website for myself.

Initially, I began the project because I didn't find any of the existing SSGs (except a select few) very appealing - most of them have very similar workflows and similar [Mustache](https://mustache.github.io)-like templating languages.
I was also constrained by the fact that I wanted to use a statically-typed language that I hadn't worked with before.

That was when I stumbled across [Hatter](https://github.com/xvxx/hatter), a rather unique templating language for Rust.
I liked it almost instantly, despite the lack of certain quality-of-life features such as editor plugins.

It took me a few days to come up with a working SSG, but more importantly, it *did* work, and the generation process was blazing fast plus memory-safe thanks to Rust.
After that, the project went through two full rewrites and a name change - I had originally intended to call it **Raven**, but later changed it when I found that there was another (unrelated) library with the same name. The current name is a reference to *Alice's Adventures in Wonderland*, keeping in theme with **Hatter**'s name.

**Jabberwock** also takes inspiration from [Metalsmith](https://metalsmith.io), a JavaScript-based SSG with an emphasis on modularity.
The modular aspect of **Jabberwock** also allows me to add or remove new features with minimal effort.