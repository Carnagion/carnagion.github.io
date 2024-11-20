+++
title = "Over-Engineering Playlist Generation"
description = "Generating playlists by solving the knapsack problem"
status = { published = "2024-11-30 12:00[Europe/London]" }
+++

Imagine you want a power metal playlist for your hour-long gym session. Or maybe something relaxing for your daily commute. Or some pirate metal [^pirate-metal] to blast as you play drinking games with your friends. Or... whatever, you get the idea. In any case, you want some kind of playlist.

Now, the normal and totally sane way of doing this would be to go through your music library and add the songs you want until you're satisfied. Maybe you even do this on a regular basis.

But there's no *fun* in doing that --- it requires *time* and *effort* to pick the best songs. Plus, you might end up with a playlist that's longer than you want it to be, and then you'd potentially have to stop a song mid-way! Can't have that.

So instead, you can write a program to automatically *generate* playlists in the most optimal way possible, such that you never get a playlist that's longer than you want, and only the most relevant songs --- according to some given metric --- are included in it.

How does one do this, you ask? By solving the knapsack problem, of course.

## The knapsack problem

The knapsack problem is a very common problem in the field of [combinatorial optimisation](https://en.wikipedia.org/wiki/Combinatorial_optimization). From [Wikipedia](https://en.wikipedia.org/wiki/Knapsack_problem):

> *Given a set of items, each with a weight and a value, determine which items to include in the collection so that the total weight is less than or equal to a given limit and the total value is as large as possible.*

I'm specifically referring to the variant known as the *0-1 knapsack problem*, wherein each item can only be included or excluded (you cannot, for example, include the same item multiple times, or include a fractional amount of an item).

### Okay, but what does this have to do with generating playlists?

You might think that generating playlists is quite simple, and that there's no need for all this fancy mathematical optimisation nonsense.

And that's true --- kind of. It's easy if you just want to shove all relevant songs into one big playlist.

But things get more complicated when you add a duration limit while wanting to maximise relevance. You can't, for example, just reverse-sort songs by relevance and then keep taking them until you hit the duration limit, because there might be a different combination that has a greater total relevance and is still under the duration limit.

If you look closely, however, this is actually just the 0-1 knapsack problem in disguise. It's easier to see if we rephrase the earlier problem statement like so:

> *Given a set of songs, each with a duration and a relevance, determine which songs to include in the playlist so that the total duration is less than or equal to a given limit and the total relevance is as large as possible.*

This description exactly matches that of the knapsack problem!

More specifically, the playlist's duration limit corresponds to the knapsack's weight limit, and a song's duration corresponds to an item's weight.

What about item value then? Well, you may have noticed that I mentioned "relevance" a couple times. What I mean by this is a quantification of how desirable a song is for the playlist you want to generate. This corresponds to an item's value in the original knapsack problem description.

For example, if you wanted a playlist full of relaxing songs, then a song's relevance is a measure of how relaxing it is. Or if you wanted a playlist of songs you hadn't listened to in a long time, then relevance is a measure of how long ago you listened to the song.

## Solving the problem

It turns out that the knapsack problem is quite hard to solve. In fact, it's [NP-complete](https://en.wikipedia.org/wiki/NP-completeness), making the playlist generation problem NP-complete too (as it is equivalent to the knapsack problem).

It is also, however, one of the most well-researched problems in the field of optimisation, so there are plenty of established approaches to solving it.

As an example [^incomplete-example], we can implement an [Iterated Local Search (ILS)](https://en.wikipedia.org/wiki/Iterated_local_search) metaheuristic. Given the following types for representing problem data and solutions:

```rs
struct Problem {
    max_duration: Duration,
    available_songs: Vec<Song>,
}

// Each `bool` represents whether the song at the given index in the `Problem` is included or not.
type Solution = Vec<bool>;
```

We can define an *evaluation function* (also known as a cost function) for quantifying how good a given solution is:

```rs
fn cost(
    solution: &Solution,
    problem: &Problem,
) -> NotNan<f64> {
    // Calculate the sum of relevances and the total duration for the playlist.
    let (relevance, duration) = solution
        .iter()
        .copied()
        .zip(problem.available_songs)
        .filter_map(|(included, song)| {
            included.then_some(song)
        })
        .fold((0.0, Duration::ZERO), |(rel, dur), song| {
            (rel + relevance(song), dur + song.duration)
        });

    // Use the negative of the duration as the cost if the duration violates our maximum duration constraint.
    let cost = if duration > problem.max_duration {
        -duration.as_secs_f64()
    } else {
        relevance
    };

    // Wrap it in a `NotNan<f64>`, since `f64`s don't have a total order.
    NotNan::new(cost).unwrap()
}
```

Finally, we can construct the metaheuristic by combining a few metaheuristic operators like so, and run it on a given problem instance to produce a solution:

```rs
let problem: Problem = /* ... */;

let flip_chance = Bernoulli::new(0.1).unwrap();
let rng = rand::thread_rng();

// Define the individual "operators" or components for the metaheuristic.
let init = init::from_individual(vec![false; problem.available_songs.len()]);
let mutate = FlipAllBits::new(flip_chance, rng);
let search = SteepestAscentBitClimb::new();
let accept = NonWorsening::new();
let stop = Iterations::new(1000);

// Create the metaheuristic itself by combining the above operators.
let mut ils = op::hint(init).then(
    op::hint(mutate)
        .then(search)
        .accept_if(accept)
        .ignore()
        .repeat_until(stop),
);

// Wrap our evaluation function into something that can be passed to `solve`.
let mut eval = eval::from_fn(cost);

// Run the metaheuristic to produce a solution.
let solution = ils.solve(&problem, &mut eval)?;
```

It's quite basic, so it's debatable whether this metaheuristic is actually good. However, the same metaheuristic performed quite well on a challenging knapsack problem instance in [one of `heur`'s examples](https://github.com/Carnagion/heur/blob/main/examples/knapsack.rs), so it might work better than expected.

Of course, metaheuristics aren't the only way to solve the knapsack problem. You could, for example, create a [mixed-integer programming](https://en.wikipedia.org/wiki/Integer_programming) model and apply an algorithm such as simplex or branch-and-bound to solve it. There's also a [dynamic programming algorithm](https://en.wikipedia.org/wiki/Knapsack_problem#Dynamic_programming_in-advance_algorithm) that solves it in $O(nW)$ time, where $n$ is the number of items and $W$ is the knapsack's weight limit.

## How useful is all this anyway?

It depends. I mean, most people probably don't care about optimising their playlists to fit under specific durations and contain only the most relevant songs according to an algorithm.

Then again, *I* found it quite neat and would definitely want it as a feature in media players, so maybe someone else will too. Who knows?

---

[^pirate-metal]: What's pirate metal, you ask? It's the most glorious sub-genre of metal ever conceived. Go read [its Wikipedia page](https://en.wikipedia.org/wiki/Pirate_metal).

[^incomplete-example]: I would usually provide complete snippets and a more detailed explanation. Unfortunately, I'm neck deep in university coursework and want to keep this post short, so I won't be providing a fully reproducible example.