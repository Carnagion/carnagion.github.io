# GDSerializer

*May 2022 - Present* **·** [GitHub](https://github.com/Carnagion/GDSerializer) **·** [NuGet](https://www.nuget.org/packages/GDSerializer)

**GDSerializer** is an XML serialization and deserialization framework developed specifically for [Godot](https://godotengine.org) applications.

It began as a custom serializer as I was dissatisfied with **Godot**'s `Export` feature, which only works with a few select C# types and promotes writing very un-idiomatic C#.

Other serialization libraries required the types to mark themselves as serializable through the use of an attribute, which was out of the question for me since I had no control over **Godot**'s source code.

With **GDSerializer**, I therefore had to take a different approach, relying heavily on [reflection](https://docs.microsoft.com/en-us/dotnet/csharp/programming-guide/concepts/reflection) to serialize all types.
Over time, I made it more abstracted and extendable - **GDSerializer** can be extended by adding custom serializers for any type, which are used instead of the default serialization algorithm when possible.

Later, I also developed [Modot](https://github.com/Carnagion/Modot) with its help, for which XML serialization was highly necessary.
**GDSerializer** is currently my second-biggest **Godot** library.