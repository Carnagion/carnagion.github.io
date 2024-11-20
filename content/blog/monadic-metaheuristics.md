+++
title = "Monadic Metaheuristics"
description = "Inventing metaheuristic combinators for fun and profit"
status = "draft"
+++

A few months ago I decided it would be a good idea to write a generic metaheuristic [^metaheuristic] framework in Rust, mostly as a fun learning experience. And as an excuse to procrastinate all the other tasks I still had to complete, but we don't talk about that.

As a result, I've spent the past few months working on this project, called [`heur`](https://github.com/Carnagion/heur). It's very much a work in progress, but here's a peek at some of its features:

- Assemble metaheuristics using combinators, much like you'd chain together parser combinators or iterator adapters
- Fully generic over problem, solution, and objective function types
- Proper ™ error handling without needing to panic or use [`anyhow`](https://docs.rs/anyhow)-style crates
- And most importantly: as efficient as a handwritten implementation!

At this point, `heur`'s overarching design is more or less finalised, so I figured I'd take a break from implementing the remaining features by writing about why I decided to make `heur` and how I arrived at its current design. ~~This is definitely not me procrastinating things again.~~

## Where did this idea come from?

My initial source of inspiration for `heur` was some code I wrote in Java about two years ago.

It started off as a couple of abstractions for my university coursework, but grew into a small mini-framework of sorts (looking back on it now, I almost certainly over-engineered the whole thing).

Unfortunately, thanks to Java's type system's limitations, the code wasn't as generalisable, efficient, or pretty as I would have liked it to be. Not to mention I was bound by coursework deadlines, which prevented me from spending an eternity ironing out the finer details (read: procrastinating) as I usually do.

In any case, I soon found myself wanting a proper framework with better abstractions. And preferably not in Java.

So, armed with the intent to Rewrite It In Rust ™, I started going through existing metaheuristic crates in hopes of finding some designs I could ~~steal~~ borrow (no pun intended). Sadly, the ecosystem here isn't very mature --- which is hardly a surprise, considering the language itself is only around 9 years old at the time of writing.

But anyway, as I was trawling through the depths of [crates.io](https://crates.io), [lib.rs](https://lib.rs), and [Are We Learning Yet?](https://www.arewelearningyet.com), filtering out ancient crates that looked like they were created by Java programmers who had worshipped inheritance all their lives, I came across an interesting project on modular metaheuristics: [MAHF](https://docs.rs/mahf).

## Metaheuristics, the modular way

In [their paper](https://opus.bibliothek.uni-augsburg.de/opus4/frontdoor/index/index/docId/103452) (yes, they published a paper), MAHF's authors describe it as "a framework for modular construction and evaluation of metaheuristics".

The keyword here is "modular". In my (possibly limited) experience, most metaheuristic or optimisation libraries typically fall into one of two categories:

- Implementations of specific algorithms (e.g. [`cobyla`](https://docs.rs/cobyla))
- Unified interfaces for solvers or algorithms which other crates can then implement for their own solvers (e.g. [`argmin`](https://docs.rs/argmin), [`genevo`](https://docs.rs/genevo), and [HyFlex](https://people.cs.nott.ac.uk/pszwj1/chesc2011/hyflex_description.html))

MAHF, however, takes a slightly different approach that's very reminiscent of the UNIX philosophy.

### Everything is a ~~file~~ component

In MAHF, metaheuristics are made up of several individual *components*, represented via the [`Component`](https://docs.rs/mahf/latest/mahf/components/trait.Component.html) trait. We can combine components together using a  convenient builder-style API.

It's easier to visualise this using an example, so consider the following pseudocode [^pseudocode] for [Iterated Local Search (ILS)](https://en.wikipedia.org/wiki/Iterated_local_search), a very basic metaheuristic:

```py
def ils(init, mutate, local_search, stop, evaluate):
    solution = init()

    while not stop(solution):
        new_solution = mutate(solution)
        new_solution = local_search(new_solution)

        if evaluate(new_solution) > evaluate(solution):
            solution = new_solution
    
    return solution
```

We initialise a solution and then repeatedly apply mutation and local search operators on it until some condition gets satisfied, accepting the modified solution each iteration only if it evaluates better than the previous one. Quite simple.

Now here's what an implementation of ILS might look like using MAHF:

```rs
let ils = Configuration::builder()
    .do_(init)
    .while_(stop, |config| {
        config
            .do_(selection::All::new())
            .do_(mutate)
            .do_(local_search)
            .evaluate()
            .update_best_individual()
            .do_(replacement::MuPlusLambda::new(1))
    })
    .build();
```

Okay, so this isn't the prettiest code in existence, but if we ignore the ugly method names as well as the selection and replacement args, then it's a fairly literal translation of the pseudocode above.

As you might have guessed, `init`, `stop`, `selection::All`, `mutate`, `local_search`, and `replacement::MuPlusLambda` are all components.

But the really fascinating bit is that the control flow constructs here are just components too! The code above actually evaluates to a [`Block`](https://docs.rs/mahf/latest/mahf/components/control_flow/struct.Block.html) component that looks something like this:

```rs
let ils = Block(vec![
    init,
    Loop {
        condition: stop,
        body: Block(vec![
            selection::All::new(),
            mutate,
            local_search,
            PopulationEvaluator::new(),
            BestIndividualUpdate::new(),
            replacement::MuPlusLambda::new(1),
        ]),
    },
]);
```

where `PopulationEvaluator` and `BestIndividualUpdate` are components inserted by [`ConfigurationBuilder::evaluate`](https://docs.rs/mahf/latest/mahf/configuration/struct.ConfigurationBuilder.html#method.evaluate) and [`ConfigurationBuilder::update_best_individual`](https://docs.rs/mahf/latest/mahf/configuration/struct.ConfigurationBuilder.html#method.update_best_individual) respectively.

To run the `ils` metaheuristic, we only need to execute this top-level `Block` component, which in turn sequentially executes its inner components. It's as shrimple as that.

### But what about runtime state?

Now this is all very convenient, but there is a slight issue: what if some components need to share information with each other? Or, since [`Component::execute`](https://docs.rs/mahf/latest/mahf/components/trait.Component.html#tymethod.execute) only provides a `&self`, what if a component needs to modify some internal state during execution?

Well, it turns out MAHF provides a solution for both of these problems, and it's called [`State`](https://docs.rs/mahf/latest/mahf/state/struct.State.html). To quote [its documentation](https://docs.rs/mahf/latest/mahf/#runtime-state):

> Components can only realize complex functionality if they can communicate freely, and the `State` offers a way to do it. The `State` is a shared blackboard where components can insert, read, write, and remove so-called [`CustomState`](https://docs.rs/mahf/latest/mahf/state/registry/trait.CustomState.html).

Being able to insert and retrieve values of any type sounds great, but how exactly does `State` achieve this?

```rs
pub struct State<'a, P> {
    registry: StateRegistry<'a>,
    marker: PhantomData<P>,
}
```

Looks like it's just a thin wrapper around [`StateRegistry`](https://docs.rs/mahf/latest/mahf/state/registry/struct.StateRegistry.html). Nothing out of the ordinary here. Let's have a peek at `StateRegistry` then:

```rs
pub struct StateRegistry<'a> {
    parent: Option<Box<StateRegistry<'a>>>,
    map: StateMap<'a>,
}
```

*Hmm*. That looks like a linked list --- and a rather confusingly implemented one at that --- trying to disguise itself as a stack. Which is already enough to make me raise my eyebrows, but let's continue to [`StateMap`](https://docs.rs/mahf/latest/mahf/state/registry/type.StateMap.html):

```rs
pub type StateMap<'a> = HashMap<TypeId, RefCell<Box<dyn CustomState<'a>>>>;
```

Aha! What we have here is a stack (again, actually a linked list) of typemaps with dynamically borrow-checked, type-erased values, aka "dynamic typing at home". That explains how `State` is able to store and access values of any type simultaneously.

However, dynamic typing isn't a silver bullet.

### Something something dynamic typing bad

Imagine you want to access a value in the `State`. You first have to look it up by hashing its corresponding [`TypeId`](https://doc.rust-lang.org/stable/std/any/struct.TypeId.html), then check that it isn't already being borrowed elsewhere, and *then* [downcast](https://doc.rust-lang.org/stable/std/boxed/struct.Box.html#method.downcast) it to its concrete type. And if the lookup fails, you have to repeat all these steps up each "level" of the stack until it succeeds, or you reach the root (at which point it fails because there's no other level to go up to).

That's a lot of steps just for accessing a value --- something that could in theory be as simple as a field access. I'm not even going to get into linked lists' horrible cache efficiency. LLVM's black magic most likely won't help us out here either, thanks to the hashing step.

I think it's reasonable to assume that all of `State`'s bookkeeping is at least an order of magnitude slower than just accessing a field or variable, though admittedly I don't have any benchmarks to back this up.

And let's not forget that any bugs in a metaheuristic using `State` will only be detected once we actually run the metaheuristic.

So just like a dynamically typed language, MAHF trades away strong guarantees and (potentially) higher base performance in exchange for a greater degree of flexibility and lower friction during development.

Which... is actually fine, in this case (hot take, I know).

MAHF accepts this trade-off because it's a *research project*. It tries to simplify research and encourage experimentation, and dynamic typing provides the kind of frictionless experience and flexible capabilities needed for that.

## Modular metaheuristics, take two

---

[^metaheuristic]: A metaheuristic is an approximate method for solving an optimisation (the mathematical kind) or machine learning problem. I'm too lazy to cover this in more detail here, so see the [Wikipedia page on metaheuristics](https://en.wikipedia.org/wiki/Metaheuristic) if you want to know more.

[^pseudocode]: Okay, it's not actually pseudocode --- it's Python without any type annotations, but that might as well be pseudocode.