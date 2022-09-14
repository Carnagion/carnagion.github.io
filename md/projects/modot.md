# Modot

*June 2022 - Present* **·** [GitHub](https://github.com/Carnagion/Modot) **·** [NuGet](https://www.nuget.org/packages/Modot)

**Modot** is a mod loader for C# [Godot](https://godotengine.org) applications, inspired by [RimWorld](https://rimworldgame.com)'s mod loading process.

I began working on parts of **Modot** back in 2021, when I first learned about and experimented with **Godot Engine**, though I took a very long break from it after a few months that produced very unsatisfactory code.

My interest in it was renewed after I created [GDSerializer](projects/gdserializer.html), and I soon had the goal of create an easy-to-use mod loading API for the vast majority of situations.

The bulk of **Modot**'s logic lies in its mod-sorting algorithm, which relies on loading metadata files containing a mod's ID, dependencies, and other information, and then sorting the metadata using a variant of [topological sort](https://en.wikipedia.org/wiki/Topological_sorting).
It took me multiple tries to write this in a way that I was satisfied with.
The current implementation is quite functional in style, making heavy use of C#'s [LINQ](https://docs.microsoft.com/en-us/dotnet/csharp/programming-guide/concepts/linq).

The mods themselves can contain XML data, C# assemblies, and XML patches, all of which are loaded at runtime and executed (in the case of assemblies and patches).
This is very similar to **RimWorld**'s mods, though **Modot** offers better XML (de)serialization due to **GDSerializer**'s highly extendable design.

These features make **Modot** one of the few libraries of its kind available for **Godot**, and over time it grew to become my biggest project so far, with over 50 stars on GitHub and hundreds of downloads on NuGet.